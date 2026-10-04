//! Linear-time energy segmentation of already decoded 16 kHz mono PCM.
use crate::config::SegmentationConfig;
use std::ops::Range;

const FRAME: usize = 160; // 10 ms at 16 kHz

pub fn split(pcm: &[f32], config: &SegmentationConfig) -> Result<Vec<Range<usize>>, String> {
    if !config.silence_relative_db.is_finite()
        || config.silence_relative_db <= 0.0
        || config
            .silence_threshold_db
            .is_some_and(|db| !db.is_finite() || db > 0.0)
        || config.silence_gap_ms == 0
        || config.max_sentence_ms < 1_000
        || config.speech_padding_ms > 1_000
        || config.min_speech_ms == 0
    {
        return Err("Invalid volume segmentation settings: use finite thresholds, positive silence/speech durations, max_sentence_ms >= 1000 and speech_padding_ms <= 1000".into());
    }
    if pcm.is_empty() {
        return Ok(Vec::new());
    }
    let mut total = 0.0;
    let mut energy = Vec::with_capacity(pcm.len().div_ceil(FRAME));
    for frame in pcm.chunks(FRAME) {
        let sum: f64 = frame.iter().map(|&sample| f64::from(sample).powi(2)).sum();
        if !sum.is_finite() {
            return Err("Decoded audio contains non-finite samples".into());
        }
        total += sum;
        energy.push(sum / frame.len() as f64);
    }
    let mut distribution = energy.clone();
    let low_index = distribution.len() / 5;
    let noise_floor = *distribution
        .select_nth_unstable_by(low_index, f64::total_cmp)
        .1;
    let high_index = distribution.len() * 95 / 100;
    let speech_level = *distribution
        .select_nth_unstable_by(high_index, f64::total_cmp)
        .1;
    let threshold = config.silence_threshold_db.map_or_else(
        || {
            (total / pcm.len() as f64 * 10_f64.powf(-f64::from(config.silence_relative_db) / 10.0))
                .max((noise_floor * 8.0).min(speech_level * 0.25))
                .max(1e-8)
        },
        |db| 10_f64.powf(f64::from(db) / 10.0),
    );
    let gap = config.silence_gap_ms.div_ceil(10) as usize;
    let min_speech = config.min_speech_ms.div_ceil(10) as usize;
    let padding = config.speech_padding_ms as usize * 16;
    let max_samples = config.max_sentence_ms.min(30_000) as usize * 16;
    let mut spans = Vec::new();
    let mut start = None;
    let mut last_voice = 0;
    let mut voiced = 0;
    for (index, &level) in energy.iter().enumerate() {
        if level > threshold {
            start.get_or_insert(index);
            last_voice = index;
            voiced += 1;
        }
        if start.is_some() && index - last_voice >= gap {
            if voiced >= min_speech {
                spans.push(start.unwrap() * FRAME..((last_voice + 1) * FRAME).min(pcm.len()));
            }
            start = None;
            voiced = 0;
        }
    }
    if let Some(start) = start {
        if voiced >= min_speech {
            spans.push(start * FRAME..((last_voice + 1) * FRAME).min(pcm.len()));
        }
    }
    let mut result = Vec::new();
    for (index, span) in spans.iter().enumerate() {
        // Padding cannot cross the middle of a pause into another utterance.
        let left = index
            .checked_sub(1)
            .map_or(0, |i| (spans[i].end + span.start) / 2);
        let right = spans
            .get(index + 1)
            .map_or(pcm.len(), |next| (span.end + next.start) / 2);
        let mut start = span.start.saturating_sub(padding).max(left);
        let end = span.end.saturating_add(padding).min(right);
        while end - start > max_samples {
            let last = (start + max_samples) / FRAME;
            let first = (start + max_samples * 2 / 3).div_ceil(FRAME);
            // Prefer a quiet frame near the duration cap; never discard voiced audio.
            let cut = (first..=last)
                .min_by(|&a, &b| energy[a].total_cmp(&energy[b]))
                .unwrap()
                * FRAME;
            result.push(start..cut);
            start = cut;
        }
        result.push(start..end);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add(pcm: &mut Vec<f32>, ms: usize, amplitude: f32) {
        pcm.extend((0..ms * 16).map(|i| if i % 2 == 0 { amplitude } else { -amplitude }));
    }

    #[test]
    fn pauses_split_with_padding_and_keep_short_internal_gaps() {
        let mut pcm = Vec::new();
        for (ms, level) in [
            (200, 0.0),
            (300, 0.2),
            (200, 0.0),
            (300, 0.2),
            (700, 0.0),
            (400, 0.2),
            (200, 0.0),
        ] {
            add(&mut pcm, ms, level);
        }
        assert_eq!(
            split(&pcm, &SegmentationConfig::default()).unwrap(),
            [0..19200, 24000..36800]
        );
    }

    #[test]
    fn quiet_audio_scales_and_noise_clicks_are_rejected() {
        let mut pcm = Vec::new();
        for (ms, level) in [
            (700, 0.0001),
            (20, 0.01),
            (700, 0.0001),
            (400, 0.01),
            (700, 0.0001),
        ] {
            add(&mut pcm, ms, level);
        }
        let spans = split(&pcm, &SegmentationConfig::default()).unwrap();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], 19520..32320);
        assert!(split(&vec![0.0; 16000], &SegmentationConfig::default())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn duration_cap_keeps_all_samples_and_partial_final_frame() {
        let pcm = vec![0.2; 100_003];
        let config = SegmentationConfig {
            max_sentence_ms: 1000,
            ..Default::default()
        };
        let spans = split(&pcm, &config).unwrap();
        assert_eq!(spans.first().unwrap().start, 0);
        assert_eq!(spans.last().unwrap().end, pcm.len());
        assert!(spans
            .iter()
            .all(|span| !span.is_empty() && span.len() <= 16000));
        assert!(spans.windows(2).all(|pair| pair[0].end == pair[1].start));
    }

    #[test]
    fn invalid_settings_and_nonfinite_audio_report_errors() {
        let config = SegmentationConfig {
            max_sentence_ms: 0,
            ..Default::default()
        };
        assert!(split(&[0.1; FRAME], &config).is_err());
        assert!(split(&[f32::NAN; FRAME], &SegmentationConfig::default()).is_err());
    }
}
