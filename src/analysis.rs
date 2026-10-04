use crate::{
    config::LoadedConfig,
    model::{SentenceSegment, WordSegment},
    workspace::WorkspacePaths,
};
use ffmpeg_next as ffmpeg;
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    ops::Range,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Instant,
};
use whisper_rs::{
    DtwMode, DtwModelPreset, FullParams, SamplingStrategy, WhisperContext,
    WhisperContextParameters, WhisperState, WhisperVadParams,
};

const WHISPER_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug)]
pub struct AnalysisResult {
    pub lesson_index: usize,
    pub extracted_audio: PathBuf,
    pub sentences: Vec<SentenceSegment>,
    pub raw_json: PathBuf,
}

#[derive(Debug)]
pub enum AnalysisProgress {
    Sentences {
        lesson_index: usize,
        sentences: Vec<SentenceSegment>,
    },
    SentenceWords {
        lesson_index: usize,
        sentence_index: usize,
        sentence: SentenceSegment,
    },
}

fn decode_media_to_pcm(source: &Path) -> Result<Vec<f32>, String> {
    ffmpeg::init().map_err(|error| format!("Could not initialize FFmpeg: {error}"))?;
    let mut input = ffmpeg::format::input(source)
        .map_err(|error| format!("Could not open {}: {error}", source.display()))?;
    let stream = input
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or_else(|| format!("{} has no audio stream", source.display()))?;
    let stream_index = stream.index();
    let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|error| format!("Could not read audio parameters: {error}"))?;
    let mut decoder = context
        .decoder()
        .audio()
        .map_err(|error| format!("Could not open audio decoder: {error}"))?;
    let input_layout = if decoder.channel_layout().is_empty() {
        ffmpeg::ChannelLayout::default(decoder.channels().into())
    } else {
        decoder.channel_layout()
    };
    let mut resampler = ffmpeg::software::resampling::Context::get(
        decoder.format(),
        input_layout,
        decoder.rate(),
        ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
        ffmpeg::ChannelLayout::MONO,
        WHISPER_SAMPLE_RATE,
    )
    .map_err(|error| format!("Could not create audio resampler: {error}"))?;
    let mut pcm = Vec::new();

    for (stream, packet) in input.packets() {
        if stream.index() == stream_index {
            decoder
                .send_packet(&packet)
                .map_err(|error| format!("Could not decode audio packet: {error}"))?;
            receive_pcm(&mut decoder, &mut resampler, &mut pcm)?;
        }
    }
    decoder
        .send_eof()
        .map_err(|error| format!("Could not finish audio decoding: {error}"))?;
    receive_pcm(&mut decoder, &mut resampler, &mut pcm)?;

    if let Some(delay) = resampler.delay() {
        let mut output = ffmpeg::frame::Audio::new(
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
            delay.output.max(1) as usize,
            ffmpeg::ChannelLayout::MONO,
        );
        output.set_rate(WHISPER_SAMPLE_RATE);
        resampler
            .flush(&mut output)
            .map_err(|error| format!("Could not flush audio resampler: {error}"))?;
        pcm.extend_from_slice(output.plane::<f32>(0));
    }
    (!pcm.is_empty())
        .then_some(pcm)
        .ok_or_else(|| format!("{} decoded to empty audio", source.display()))
}

fn receive_pcm(
    decoder: &mut ffmpeg::decoder::Audio,
    resampler: &mut ffmpeg::software::resampling::Context,
    pcm: &mut Vec<f32>,
) -> Result<(), String> {
    let mut decoded = ffmpeg::frame::Audio::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let capacity = ((decoded.samples() as u64 * WHISPER_SAMPLE_RATE as u64)
            .div_ceil(decoded.rate() as u64)
            + 256) as usize;
        let mut output = ffmpeg::frame::Audio::new(
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
            capacity,
            ffmpeg::ChannelLayout::MONO,
        );
        output.set_rate(WHISPER_SAMPLE_RATE);
        resampler
            .run(&decoded, &mut output)
            .map_err(|error| format!("Could not resample audio: {error}"))?;
        pcm.extend_from_slice(output.plane::<f32>(0));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct WordDraft {
    label: String,
    start_ms: u64,
    end_ms: u64,
    alignment_ms: Option<u64>,
    probabilities: Vec<f32>,
}

pub fn analyze_media_with_progress(
    lesson_index: usize,
    source: &Path,
    workspace: &WorkspacePaths,
    config: &LoadedConfig,
    mut progress: impl FnMut(AnalysisProgress),
) -> Result<AnalysisResult, String> {
    workspace
        .create()
        .map_err(|error| format!("Could not create media workspace: {error}"))?;
    let model = config.whisper_model().ok_or_else(|| {
        format!(
            "Whisper model is not available. Check {}",
            config.path.display()
        )
    })?;

    let started = Instant::now();
    let pcm = decode_media_to_pcm(source)?;
    let decoded = started.elapsed();
    if config.values.segmentation.use_volume_segmentation {
        return analyze_volume_segments(
            lesson_index,
            source,
            workspace,
            config,
            &pcm,
            decoded,
            progress,
        );
    }
    let mut context_params = WhisperContextParameters::default();
    context_params.dtw_parameters.mode = DtwMode::ModelPreset {
        model_preset: DtwModelPreset::Base,
    };
    let context = WhisperContext::new_with_params(&model, context_params)
        .map_err(|error| format!("Could not load Whisper model: {error}"))?;
    let mut state = context
        .create_state()
        .map_err(|error| format!("Could not create Whisper state: {error}"))?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(&config.values.segmentation.language));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    params.set_token_timestamps(true);
    params.set_split_on_word(true);
    params.set_max_len(1);
    if config.values.segmentation.use_vad {
        let vad = config.vad_model().ok_or_else(|| {
            format!(
                "VAD model is not available. Check {}",
                config.path.display()
            )
        })?;
        let vad = vad.to_string_lossy();
        let mut vad_params = WhisperVadParams::new();
        vad_params.set_min_silence_duration(
            config
                .values
                .segmentation
                .vad_min_silence_ms
                .min(i32::MAX as u64) as i32,
        );
        vad_params.set_speech_pad(
            config
                .values
                .segmentation
                .speech_padding_ms
                .min(i32::MAX as u64) as i32,
        );
        params.set_vad_model_path(Some(&vad));
        params.set_vad_params(vad_params);
        params.enable_vad(true);
    }
    state
        .full(params, &pcm)
        .map_err(|error| format!("Whisper segmentation failed: {error}"))?;
    let aligned_words = collect_words(&state)?;
    let mut sentences = group_sentences(
        aligned_words.clone(),
        config.values.segmentation.silence_gap_ms,
        config.values.segmentation.max_sentence_ms,
    );
    if sentences.is_empty() {
        return Err("Whisper completed but did not return any speech segments".into());
    }
    let mut offsets = Vec::with_capacity(sentences.len() + 1);
    offsets.push(0);
    for sentence in &sentences {
        offsets.push(offsets.last().unwrap() + sentence.words.len());
    }
    progress(AnalysisProgress::Sentences {
        lesson_index,
        sentences: sentences
            .iter()
            .cloned()
            .map(|mut sentence| {
                sentence.words.clear();
                sentence
            })
            .collect(),
    });
    drop(state);
    let media_end_ms = pcm.len() as u64 * 1_000 / WHISPER_SAMPLE_RATE as u64;
    let limits: Vec<_> = sentences
        .iter()
        .skip(1)
        .map(|sentence| sentence.start_ms.min(media_end_ms))
        .chain(std::iter::once(media_end_ms))
        .collect();
    refine_word_boundaries(
        &context,
        &pcm,
        &mut sentences,
        &config.values.segmentation.language,
        |sentence_index, sentence| {
            apply_word_alignment(
                sentence,
                &aligned_words[offsets[sentence_index]..offsets[sentence_index + 1]],
                config.values.segmentation.word_boundary_padding_ms,
                limits[sentence_index],
            );
            progress(AnalysisProgress::SentenceWords {
                lesson_index,
                sentence_index,
                sentence: sentence.clone(),
            });
        },
    )?;
    let raw_json = workspace.segmentations.join("segments.json");
    fs::write(&raw_json, segmentation_json(&sentences)?).map_err(|error| error.to_string())?;
    Ok(AnalysisResult {
        lesson_index,
        extracted_audio: source.to_path_buf(),
        sentences,
        raw_json,
    })
}

fn analyze_volume_segments(
    lesson_index: usize,
    source: &Path,
    workspace: &WorkspacePaths,
    config: &LoadedConfig,
    pcm: &[f32],
    decode_time: std::time::Duration,
    mut progress: impl FnMut(AnalysisProgress),
) -> Result<AnalysisResult, String> {
    let started = Instant::now();
    let settings = &config.values.segmentation;
    let spans = crate::silence::split(pcm, settings)?;
    let split_time = started.elapsed();
    if spans.is_empty() {
        return Err("No speech exceeded the configured volume threshold".into());
    }
    let mut sentences: Vec<_> = spans
        .iter()
        .enumerate()
        .map(|(i, span)| SentenceSegment {
            id: format!("s{:03}", i + 1),
            text: String::new(),
            start_ms: span.start as u64 / 16,
            end_ms: span.end as u64 / 16,
            enabled: true,
            confidence: None,
            words: Vec::new(),
        })
        .collect();
    progress(AnalysisProgress::Sentences {
        lesson_index,
        sentences: sentences.clone(),
    });

    let model = config
        .whisper_model()
        .ok_or_else(|| "Whisper model is not available".to_string())?;
    let mut context_params = WhisperContextParameters::default();
    context_params.dtw_parameters.mode = DtwMode::ModelPreset {
        model_preset: DtwModelPreset::Base,
    };
    let context = WhisperContext::new_with_params(&model, context_params)
        .map_err(|error| format!("Could not load Whisper model: {error}"))?;
    let cpu_budget = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8);
    let workers = settings
        .whisper_workers
        .max(1)
        .min(cpu_budget)
        .min(spans.len());
    let threads = if settings.whisper_threads == 0 {
        cpu_budget / workers
    } else {
        settings.whisper_threads.min(cpu_budget / workers)
    }
    .max(1);
    // Allocate states sequentially; model weights and PCM remain shared read-only.
    let states = (0..workers)
        .map(|_| {
            context
                .create_state()
                .map_err(|error| format!("Could not create Whisper state: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (sender, receiver) = mpsc::channel();
    let next = AtomicUsize::new(0);
    let cancelled = AtomicBool::new(false);
    let recognition_started = Instant::now();
    thread::scope(|scope| -> Result<(), String> {
        let mut handles = Vec::new();
        for mut state in states {
            let sender = sender.clone();
            let next = &next;
            let cancelled = &cancelled;
            let spans = &spans;
            handles.push(scope.spawn(move || {
                while !cancelled.load(Ordering::Relaxed) {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(span) = spans.get(index) else {
                        break;
                    };
                    let result = transcribe_span(&mut state, pcm, span, index, settings, threads);
                    if result.is_err() {
                        cancelled.store(true, Ordering::Relaxed);
                    }
                    if sender.send((index, result)).is_err() {
                        break;
                    }
                }
            }));
        }
        drop(sender);
        let mut pending = BTreeMap::new();
        let mut completed = 0;
        let mut failure = None;
        for (index, result) in receiver {
            match result {
                Ok(sentence) => {
                    pending.insert(index, sentence);
                }
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
            while let Some(sentence) = pending.remove(&completed) {
                sentences[completed] = sentence.clone();
                progress(AnalysisProgress::SentenceWords {
                    lesson_index,
                    sentence_index: completed,
                    sentence,
                });
                completed += 1;
            }
        }
        for handle in handles {
            if handle.join().is_err() {
                failure.get_or_insert("Whisper worker panicked".into());
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        if completed != spans.len() {
            return Err("Whisper workers did not complete every audio segment".into());
        }
        Ok(())
    })?;
    // A pause span can contain several grammatical sentences, especially with
    // background music. Split punctuation after recognition without another pass.
    sentences = group_sentences(
        sentences
            .into_iter()
            .flat_map(|sentence| {
                sentence.words.into_iter().map(|word| WordDraft {
                    label: word.label,
                    start_ms: word.start_ms,
                    end_ms: word.end_ms,
                    alignment_ms: None,
                    probabilities: word.probability.into_iter().collect(),
                })
            })
            .collect(),
        settings.silence_gap_ms,
        settings.max_sentence_ms,
    );
    if sentences.is_empty() {
        return Err("Whisper completed but did not return any speech segments".into());
    }
    let raw_json = workspace.segmentations.join("segments.json");
    fs::write(&raw_json, segmentation_json(&sentences)?).map_err(|error| error.to_string())?;
    eprintln!("Volume segmentation: decode={decode_time:?}, split={split_time:?}, recognition={:?}, segments={}, workers={workers}, threads/worker={threads}", recognition_started.elapsed(), spans.len());
    Ok(AnalysisResult {
        lesson_index,
        extracted_audio: source.to_path_buf(),
        sentences,
        raw_json,
    })
}

fn transcribe_span(
    state: &mut WhisperState,
    pcm: &[f32],
    span: &Range<usize>,
    index: usize,
    settings: &crate::config::SegmentationConfig,
    threads: usize,
) -> Result<SentenceSegment, String> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(&settings.language));
    params.set_n_threads(threads as i32);
    params.set_no_context(true);
    // Each energy span is already a bounded utterance. A single decoding
    // segment avoids tiny timestamp-induced DTW windows in whisper.cpp.
    params.set_single_segment(true);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    params.set_token_timestamps(true);
    params.set_split_on_word(true);
    params.set_max_len(1);
    let short_audio;
    let audio = if span.len() < WHISPER_SAMPLE_RATE as usize {
        short_audio = {
            let mut samples = pcm[span.clone()].to_vec();
            samples.resize(WHISPER_SAMPLE_RATE as usize, 0.0);
            samples
        };
        &short_audio[..]
    } else {
        &pcm[span.clone()]
    };
    state
        .full(params, audio)
        .map_err(|error| format!("Whisper failed on segment {}: {error}", index + 1))?;
    let start_ms = span.start as u64 / 16;
    let end_ms = (span.end as u64 / 16).max(start_ms + 1);
    let mut words = collect_words(state)?;
    let transcript = words
        .iter()
        .map(|word| word.label.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if transcript.starts_with('(') && transcript.ends_with(')') {
        words.clear(); // Non-verbal annotations such as "(soft music)".
    }
    for word in &mut words {
        word.start_ms = (start_ms + word.start_ms).clamp(start_ms, end_ms - 1);
        word.end_ms = (start_ms + word.end_ms).clamp(word.start_ms + 1, end_ms);
        word.alignment_ms = word.alignment_ms.map(|ms| start_ms + ms);
    }
    let aligned = words.clone();
    let mut result = Vec::new();
    push_sentence(&mut result, &mut words);
    let mut sentence = result.pop().unwrap_or_else(|| SentenceSegment {
        id: String::new(),
        text: String::new(),
        start_ms,
        end_ms,
        enabled: false,
        confidence: None,
        words: Vec::new(),
    });
    sentence.id = format!("s{:03}", index + 1);
    sentence.start_ms = sentence.start_ms.clamp(start_ms, end_ms - 1);
    sentence.end_ms = sentence.end_ms.clamp(sentence.start_ms + 1, end_ms);
    apply_word_alignment(
        &mut sentence,
        &aligned,
        settings.word_boundary_padding_ms,
        end_ms,
    );
    for word in &mut sentence.words {
        word.start_ms = word.start_ms.clamp(sentence.start_ms, sentence.end_ms - 1);
        word.end_ms = word.end_ms.clamp(word.start_ms + 1, sentence.end_ms);
    }
    Ok(sentence)
}

pub(crate) fn segmentation_json(sentences: &[SentenceSegment]) -> Result<String, String> {
    let document: Vec<_> = sentences
        .iter()
        .map(|sentence| {
            json!({
                "id": sentence.id,
                "text": sentence.text,
                "start": timestamp(sentence.start_ms),
                "end": timestamp(sentence.end_ms),
                "enabled": sentence.enabled,
                "confidence": sentence.confidence,
                "words": sentence.words.iter().map(|word| json!({
                    "id": word.id,
                    "label": word.label,
                    "start": timestamp(word.start_ms),
                    "end": timestamp(word.end_ms),
                    "probability": word.probability,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&document).map_err(|error| error.to_string())
}

fn timestamp(ms: u64) -> String {
    format!(
        "{:02}:{:02}:{:02},{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1_000 % 60,
        ms % 1_000
    )
}

fn collect_words(state: &WhisperState) -> Result<Vec<WordDraft>, String> {
    let mut words = Vec::new();
    for segment in state.as_iter() {
        let text = segment
            .to_str_lossy()
            .map_err(|error| format!("Could not read Whisper segment: {error}"))?;
        let label = text.trim();
        if label.is_empty() || is_special(label) {
            continue;
        }
        let mut probabilities = Vec::new();
        let mut alignment_ms = None;
        for index in 0..segment.n_tokens() {
            let token = segment
                .get_token(index)
                .ok_or_else(|| format!("Whisper returned an invalid token index: {index}"))?;
            let text = token
                .to_str_lossy()
                .map_err(|error| format!("Could not read Whisper token: {error}"))?;
            let trimmed = text.trim();
            if trimmed.is_empty() || is_special(trimmed) {
                continue;
            }
            probabilities.push(token.token_probability());
            let timestamp = token.token_data().t_dtw;
            if timestamp >= 0 && trimmed.chars().any(char::is_alphanumeric) {
                alignment_ms = Some(timestamp as u64 * 10);
            }
        }
        // max_len=1 + split_on_word already produces whole-word segments.
        // Keep each segment's text and boundaries together instead of rebuilding
        // the word from token timestamps after Whisper has split the result.
        let start_ms = segment.start_timestamp().max(0) as u64 * 10;
        words.push(WordDraft {
            label: label.into(),
            start_ms,
            end_ms: (segment.end_timestamp().max(0) as u64 * 10).max(start_ms + 1),
            alignment_ms,
            probabilities,
        });
    }
    Ok(words)
}

fn group_sentences(
    words: Vec<WordDraft>,
    silence_gap_ms: u64,
    max_sentence_ms: u64,
) -> Vec<SentenceSegment> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for word in words {
        let split_before = current.last().is_some_and(|previous: &WordDraft| {
            word.start_ms.saturating_sub(previous.end_ms) >= silence_gap_ms
                || word.end_ms.saturating_sub(current[0].start_ms) > max_sentence_ms
        });
        if split_before {
            push_sentence(&mut result, &mut current);
        }
        let closes_sentence = ends_sentence(&word.label);
        current.push(word);
        if closes_sentence {
            push_sentence(&mut result, &mut current);
        }
    }
    push_sentence(&mut result, &mut current);
    result
}

fn push_sentence(result: &mut Vec<SentenceSegment>, words: &mut Vec<WordDraft>) {
    if words.is_empty() {
        return;
    }
    align_words(words);
    let sentence_index = result.len() + 1;
    let start_ms = words.first().map(|word| word.start_ms).unwrap_or_default();
    let end_ms = words.last().map(|word| word.end_ms).unwrap_or(start_ms + 1);
    let text = words
        .iter()
        .map(|word| word.label.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let probabilities: Vec<f32> = words
        .iter()
        .flat_map(|word| word.probabilities.iter().copied())
        .collect();
    let confidence = (!probabilities.is_empty())
        .then(|| probabilities.iter().sum::<f32>() / probabilities.len() as f32);
    let normalized_words = words
        .drain(..)
        .enumerate()
        .map(|(index, word)| WordSegment {
            id: format!("w{:03}", index + 1),
            label: word.label,
            start_ms: word.start_ms,
            end_ms: word.end_ms,
            probability: (!word.probabilities.is_empty())
                .then(|| word.probabilities.iter().sum::<f32>() / word.probabilities.len() as f32),
        })
        .collect();
    result.push(SentenceSegment {
        id: format!("s{sentence_index:03}"),
        text,
        start_ms,
        end_ms,
        enabled: true,
        confidence,
        words: normalized_words,
    });
}

fn align_words(words: &mut [WordDraft]) {
    let mut previous_anchor = None;
    for word in words {
        let Some(anchor) = word.alignment_ms else {
            continue;
        };
        if anchor.abs_diff(word.end_ms) > 350 {
            let duration = word.end_ms.saturating_sub(word.start_ms).clamp(180, 500);
            word.start_ms = previous_anchor
                .map(|previous: u64| previous.saturating_sub(40))
                .unwrap_or_else(|| anchor.saturating_sub(duration));
            word.end_ms = (anchor + 80).max(word.start_ms + 180);
        }
        previous_anchor = Some(anchor);
    }
}

fn refine_word_boundaries(
    context: &WhisperContext,
    pcm: &[f32],
    sentences: &mut [SentenceSegment],
    language: &str,
    mut completed: impl FnMut(usize, &mut SentenceSegment),
) -> Result<(), String> {
    let media_end_ms = pcm.len() as u64 * 1_000 / WHISPER_SAMPLE_RATE as u64;
    let mut state = context
        .create_state()
        .map_err(|error| format!("Could not create Whisper alignment state: {error}"))?;
    for (sentence_index, sentence) in sentences.iter_mut().enumerate() {
        let slice_start_ms = sentence.start_ms.saturating_sub(100);
        let slice_end_ms = (sentence.end_ms + 300).min(media_end_ms);
        let start = (slice_start_ms * WHISPER_SAMPLE_RATE as u64 / 1_000) as usize;
        let end = (slice_end_ms * WHISPER_SAMPLE_RATE as u64 / 1_000) as usize;
        if end <= start || end > pcm.len() {
            completed(sentence_index, sentence);
            continue;
        }

        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(true);
        params.set_split_on_word(true);
        params.set_max_len(1);
        state
            .full(params, &pcm[start..end])
            .map_err(|error| format!("Whisper word alignment failed: {error}"))?;
        let refined = collect_words(&state)?;
        if refined.len() != sentence.words.len()
            || refined
                .iter()
                .zip(&sentence.words)
                .any(|(left, right)| normalized_word(&left.label) != normalized_word(&right.label))
        {
            completed(sentence_index, sentence);
            continue;
        }
        for (word, refined) in sentence.words.iter_mut().zip(refined) {
            word.start_ms = (slice_start_ms + refined.start_ms).min(slice_end_ms - 1);
            word.end_ms = (slice_start_ms + refined.end_ms).clamp(word.start_ms + 1, slice_end_ms);
        }
        sentence.start_ms = sentence.words[0].start_ms;
        sentence.end_ms = sentence.words.last().unwrap().end_ms;
        completed(sentence_index, sentence);
    }
    Ok(())
}

fn normalized_word(word: &str) -> String {
    word.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn apply_word_alignment(
    sentence: &mut SentenceSegment,
    aligned: &[WordDraft],
    padding_ms: u64,
    limit_ms: u64,
) {
    let Some(anchors) = aligned
        .iter()
        .map(|word| word.alignment_ms)
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    if anchors.len() != sentence.words.len() || anchors.windows(2).any(|pair| pair[0] >= pair[1]) {
        return;
    }
    let mut boundaries = vec![sentence.start_ms];
    for (index, &anchor) in anchors.iter().enumerate() {
        // whisper.cpp's DTW transition marks the end of the lexical token.
        // Punctuation must not extend it into the following pause. Limit the
        // release tail by the next word's spacing so short words stay distinct.
        // ponytail: this tail is heuristic; use phoneme alignment for tighter coarticulation cuts.
        let tail = anchors
            .get(index + 1)
            .map_or(padding_ms, |next| padding_ms.min((next - anchor) / 4));
        // Local segment estimates may end before the final spoken word. Only
        // the next sentence/media end can bound the original acoustic anchors.
        let end = anchor.saturating_add(tail).min(limit_ms);
        if end <= *boundaries.last().unwrap() {
            return;
        }
        boundaries.push(end);
    }
    // Keep a usable local end: a late DTW anchor can land in trailing silence.
    // Only bypass it when clipping would collapse the last word and discard
    // the entire alignment (as happened to "Hope you feel better soon").
    if boundaries
        .iter()
        .rev()
        .nth(1)
        .is_some_and(|&end| end < sentence.end_ms)
    {
        for end in boundaries.iter_mut().skip(1) {
            *end = (*end).min(sentence.end_ms);
        }
    } else if anchors
        .windows(2)
        .any(|pair| pair[1] - pair[0] > sentence.end_ms.saturating_sub(sentence.start_ms))
    {
        // A gap between words longer than the local utterance is not reliable
        // evidence for extending it. Keep the local pass instead of its silence.
        return;
    }
    for (word, range) in sentence.words.iter_mut().zip(boundaries.windows(2)) {
        word.start_ms = range[0];
        word.end_ms = range[1];
    }
    sentence.end_ms = sentence
        .end_ms
        .max(*boundaries.last().unwrap())
        .min(limit_ms);
}

fn ends_sentence(text: &str) -> bool {
    text.ends_with(['.', '?', '!', '。', '？', '！'])
}
fn is_special(text: &str) -> bool {
    (text.starts_with('[') && text.ends_with(']'))
        || (text.starts_with("<|") && text.ends_with("|>"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_starts_a_new_sentence() {
        let words = vec![
            WordDraft {
                label: "Hello".into(),
                start_ms: 0,
                end_ms: 400,
                alignment_ms: None,
                probabilities: vec![],
            },
            WordDraft {
                label: "again".into(),
                start_ms: 1_200,
                end_ms: 1_500,
                alignment_ms: None,
                probabilities: vec![],
            },
        ];
        assert_eq!(group_sentences(words, 650, 15_000).len(), 2);
    }

    #[test]
    fn segmentation_json_uses_timestamp_strings_not_milliseconds() {
        let sentences = group_sentences(
            vec![WordDraft {
                label: "Hello.".into(),
                start_ms: 190,
                end_ms: 1_230,
                alignment_ms: None,
                probabilities: vec![],
            }],
            650,
            15_000,
        );

        let json = segmentation_json(&sentences).unwrap();

        assert!(json.contains("\"start\": \"00:00:00,190\""));
        assert!(json.contains("\"end\": \"00:00:01,230\""));
        assert!(!json.contains("start_ms"));
        assert!(!json.contains("end_ms"));
    }

    #[test]
    fn ffmpeg_decodes_mp3_to_whisper_pcm() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/first snowfall.mp3");
        let pcm = decode_media_to_pcm(&source).unwrap();
        assert!(pcm.len() > 16_000);
        assert!(pcm.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn ffmpeg_decodes_mp4_to_whisper_pcm() {
        let source = LoadedConfig::load()
            .repo_root
            .join("How are you - Relax - Phrases ( lingoneo.org ).mp4");
        if !source.is_file() {
            return;
        } // Optional external fixture; MP3 is bundled.
        let pcm = decode_media_to_pcm(&source).unwrap();
        assert!(pcm.len() > 16_000);
        assert!(pcm.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    #[ignore = "requires the local Whisper model"]
    fn whisper_rs_segments_real_mp3() {
        let config = LoadedConfig::load();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/first snowfall.mp3");
        let root =
            std::env::temp_dir().join(format!("bibiparrot-analysis-e2e-{}", std::process::id()));
        let workspace = WorkspacePaths::from_root(root, &config.values.workspace);
        let mut progress = Vec::new();
        let result = analyze_media_with_progress(0, &source, &workspace, &config, |update| {
            progress.push(update)
        })
        .unwrap();
        let AnalysisProgress::Sentences {
            lesson_index,
            sentences,
        } = &progress[0]
        else {
            panic!("sentence progress must be published first")
        };
        assert_eq!(*lesson_index, 0);
        assert!(sentences.iter().all(|sentence| sentence.words.is_empty()));
        let completed: Vec<_> = progress
            .iter()
            .filter_map(|update| match update {
                AnalysisProgress::SentenceWords { sentence_index, .. } => Some(*sentence_index),
                AnalysisProgress::Sentences { .. } => None,
            })
            .collect();
        assert_eq!(completed, (0..sentences.len()).collect::<Vec<_>>());
        assert_eq!(result.extracted_audio, source);
        assert!(result.raw_json.is_file());
        assert_eq!(
            result.raw_json.file_name().and_then(|name| name.to_str()),
            Some("segments.json")
        );
        assert!(!result.sentences.is_empty());
        assert!(result
            .sentences
            .iter()
            .any(|sentence| !sentence.words.is_empty()));
        let words: Vec<_> = result.sentences.iter().flat_map(|s| &s.words).collect();
        assert_eq!(
            words.iter().map(|w| w.label.as_str()).collect::<Vec<_>>(),
            ["Today", "is", "November", "26th."]
        );
        // Independent CTC spans, including the spoken form of "26th".
        for (word, (start, end)) in
            words
                .iter()
                .zip([(261, 642), (843, 903), (983, 1424), (1545, 2166)])
        {
            let overlap = word
                .end_ms
                .min(end)
                .saturating_sub(word.start_ms.max(start));
            assert!(
                overlap * 100 >= (end - start) * 70,
                "{word:?} misses speech at {start}..{end}"
            );
        }
    }

    #[test]
    #[ignore = "requires the local Whisper model"]
    fn whisper_rs_segments_real_mp4() {
        let config = LoadedConfig::load();
        let source = config
            .repo_root
            .join("How are you - Relax - Phrases ( lingoneo.org ).mp4");
        let root = std::env::temp_dir().join(format!(
            "bibiparrot-analysis-mp4-e2e-{}",
            std::process::id()
        ));
        let workspace = WorkspacePaths::from_root(root, &config.values.workspace);
        let result = analyze_media_with_progress(0, &source, &workspace, &config, |_| {}).unwrap();
        assert_eq!(result.extracted_audio, source);
        assert!(result.raw_json.is_file());
        assert!(!result.sentences.is_empty());
        assert!(result
            .sentences
            .iter()
            .any(|sentence| !sentence.words.is_empty()));
        let first = &result.sentences[0];
        assert_eq!(first.text, "Hi, how are you?");
        let last = result.sentences.last().unwrap();
        assert_eq!(last.text, "Hope you feel better soon.");
        // Independent CTC: the final 'soon' occupies 216.394..216.735 s.
        assert!(
            last.end_ms >= 216_735,
            "Last sentence cuts off its speech: {last:?}"
        );
        // An imprecise final DTW anchor must not extend already valid ranges
        // into long pauses. Only repair ranges that collapse word alignment.
        assert!(result.sentences[2].end_ms <= 22_500);
        assert!(first.words[0].start_ms >= 4_000, "{first:?}");
        assert!(first.words.last().unwrap().end_ms <= 6_500, "{first:?}");
        // Independent wav2vec2 CTC acoustic spans; see tests/audit_word_audio.py.
        // These references must not be regenerated from Whisper's output.
        let references = [
            vec![
                ("hi", 4465, 4666),
                ("how", 5110, 5231),
                ("are", 5312, 5413),
                ("you", 5473, 5574),
            ],
            vec![
                ("hey", 12364, 12566),
                ("hows", 12909, 13110),
                ("it", 13211, 13251),
                ("going", 13312, 13735),
            ],
            vec![
                ("what", 20444, 20565),
                ("are", 20646, 20746),
                ("you", 20807, 20907),
                ("up", 21048, 21109),
                ("to", 21189, 21370),
            ],
            vec![
                ("hope", 215370, 215531),
                ("you", 215591, 215671),
                ("feel", 215771, 215972),
                ("better", 216053, 216293),
                ("soon", 216394, 216735),
            ],
        ];
        assert!(result.sentences.len() >= references.len());
        for (sentence, reference) in result
            .sentences
            .iter()
            .take(3)
            .chain(result.sentences.last())
            .zip(references)
        {
            assert_eq!(sentence.words.len(), reference.len(), "{sentence:?}");
            for (word, &(label, start, end)) in sentence.words.iter().zip(&reference) {
                assert_eq!(normalized_word(&word.label), label);
                let overlap = word
                    .end_ms
                    .min(end)
                    .saturating_sub(word.start_ms.max(start));
                assert!(
                    overlap * 100 >= (end - start) * 70,
                    "{word:?} misses its independently aligned speech at {start}..{end}"
                );
                for &(other, other_start, other_end) in &reference {
                    if other != label {
                        let overlap = word
                            .end_ms
                            .min(other_end)
                            .saturating_sub(word.start_ms.max(other_start));
                        assert!(
                            overlap * 100 <= (other_end - other_start) * 30,
                            "{word:?} contains another word's speech: {other}"
                        );
                    }
                }
            }
        }
    }
}
