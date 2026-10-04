use crate::workspace::WorkspacePaths;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Audio,
    Video,
}

impl MediaKind {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "mp4" | "mkv" | "webm" | "mov" | "avi" | "m4v" => Self::Video,
            _ => Self::Audio,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Audio => "AUDIO",
            Self::Video => "VIDEO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordSegment {
    pub id: String,
    pub label: String,
    pub start_ms: u64,
    pub end_ms: u64,
    #[serde(default)]
    pub probability: Option<f32>,
}

impl WordSegment {
    pub fn playback_range(&self, padding_ms: u64, duration_ms: u64) -> (u64, u64) {
        let end = self
            .end_ms
            .saturating_add(padding_ms)
            .min(duration_ms.max(1));
        (
            self.start_ms
                .saturating_sub(padding_ms)
                .min(end.saturating_sub(1)),
            end,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceSegment {
    pub id: String,
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub enabled: bool,
    #[serde(default)]
    pub confidence: Option<f32>,
    pub words: Vec<WordSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub source_path: String,
    pub playback_path: String,
    pub workspace_path: String,
    pub kind: MediaKind,
    pub duration_ms: u64,
    pub sentences: Vec<SentenceSegment>,
    pub markdown: String,
    #[serde(default)]
    pub analyzed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddLessonOutcome {
    Added(usize),
    Existing(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    Off,
    #[serde(alias = "Media", alias = "Sentence", alias = "Word")]
    Loop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub lessons: Vec<Lesson>,
    pub active_lesson: usize,
    pub selected_sentence: usize,
    pub selected_word: Option<usize>,
    pub loop_mode: LoopMode,
    pub playback_rate: f32,
    pub volume: u8,
}

impl Project {
    pub fn sample(sample_path: &Path, workspace: &WorkspacePaths) -> Self {
        let duration_ms = 2_300;
        let words = [
            ("Today", 230, 470),
            ("is", 470, 660),
            ("November", 660, 1_420),
            ("26th.", 1_420, 2_200),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (label, start_ms, end_ms))| WordSegment {
            id: format!("w{:03}", index + 1),
            label: label.into(),
            start_ms,
            end_ms,
            probability: None,
        })
        .collect();
        let sentence = SentenceSegment {
            id: "s001".into(),
            text: "Today is November 26th.".into(),
            start_ms: 230,
            end_ms: 2_200,
            enabled: true,
            confidence: None,
            words,
        };
        let markdown = "# First snowfall\n\n## Dictation\n\nI heard [Today is November 26th.](bibi://sentence/s001)\n\nPlay a segment, select text, then press **Link current segment**. Double-click blue linked text to replay it.\n".into();
        Self {
            version: 6,
            lessons: vec![Lesson {
                id: "lesson-first-snowfall".into(),
                title: "First snowfall".into(),
                source_path: sample_path.to_string_lossy().into_owned(),
                playback_path: sample_path.to_string_lossy().into_owned(),
                workspace_path: workspace.root.to_string_lossy().into_owned(),
                kind: MediaKind::Audio,
                duration_ms,
                sentences: vec![sentence],
                markdown,
                analyzed: false,
            }],
            active_lesson: 0,
            selected_sentence: 0,
            selected_word: Some(0),
            loop_mode: LoopMode::Loop,
            playback_rate: 1.0,
            volume: 80,
        }
    }

    pub fn active(&self) -> Option<&Lesson> {
        self.lessons.get(self.active_lesson)
    }
    pub fn active_mut(&mut self) -> Option<&mut Lesson> {
        self.lessons.get_mut(self.active_lesson)
    }
    pub fn selected_sentence(&self) -> Option<&SentenceSegment> {
        self.active()?.sentences.get(self.selected_sentence)
    }
    pub fn select_lesson(&mut self, index: usize) {
        if index >= self.lessons.len() {
            return;
        }
        self.active_lesson = index;
        self.selected_sentence = 0;
        self.selected_word = self
            .active()
            .and_then(|lesson| lesson.sentences.first())
            .and_then(|sentence| (!sentence.words.is_empty()).then_some(0));
    }

    pub fn remove_lesson(&mut self, index: usize) -> Option<Lesson> {
        if index >= self.lessons.len() {
            return None;
        }
        let removed_active = index == self.active_lesson;
        let removed = self.lessons.remove(index);

        if self.lessons.is_empty() {
            self.active_lesson = 0;
            self.selected_sentence = 0;
            self.selected_word = None;
        } else if removed_active {
            self.select_lesson(index.min(self.lessons.len() - 1));
        } else if index < self.active_lesson {
            self.active_lesson -= 1;
        }

        Some(removed)
    }

    pub fn lesson_index_for_path(&self, path: &Path) -> Option<usize> {
        let key = media_path_key(path);
        self.lessons
            .iter()
            .position(|lesson| media_path_key(Path::new(&lesson.source_path)) == key)
    }

    pub fn add_lesson(
        &mut self,
        path: &Path,
        duration_ms: u64,
        workspace: &WorkspacePaths,
    ) -> AddLessonOutcome {
        if let Some(index) = self.lesson_index_for_path(path) {
            self.select_lesson(index);
            return AddLessonOutcome::Existing(index);
        }
        let title = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Untitled media")
            .to_string();
        let id = format!("lesson-{:03}", self.lessons.len() + 1);
        let sentence = SentenceSegment {
            id: "s001".into(),
            text: "Run intelligent segmentation to transcribe this media.".into(),
            start_ms: 0,
            end_ms: duration_ms.max(1),
            enabled: true,
            confidence: None,
            words: Vec::new(),
        };
        self.lessons.push(Lesson {
            id,
            title: title.clone(),
            source_path: path.to_string_lossy().into_owned(),
            playback_path: path.to_string_lossy().into_owned(),
            workspace_path: workspace.root.to_string_lossy().into_owned(),
            kind: MediaKind::from_path(path),
            duration_ms: duration_ms.max(1),
            sentences: vec![sentence],
            markdown: format!("# {title}\n\n## Dictation\n\n"),
            analyzed: false,
        });
        let index = self.lessons.len() - 1;
        self.select_lesson(index);
        AddLessonOutcome::Added(index)
    }

    pub fn apply_analysis(
        &mut self,
        lesson_index: usize,
        playback_path: PathBuf,
        sentences: Vec<SentenceSegment>,
    ) {
        if let Some(lesson) = self.lessons.get_mut(lesson_index) {
            lesson.playback_path = playback_path.to_string_lossy().into_owned();
            lesson.sentences = sentences;
            lesson.analyzed = true;
        }
        if self.active_lesson == lesson_index {
            self.selected_sentence = 0;
            self.selected_word = self
                .selected_sentence()
                .and_then(|sentence| (!sentence.words.is_empty()).then_some(0));
        }
    }

    pub fn normalize(&mut self) {
        if self.version < 6 {
            // Refresh corrected sentence/word boundaries; retain dictation and
            // existing IDs until reanalysis succeeds.
            for lesson in &mut self.lessons {
                lesson.analyzed = false;
            }
            self.version = 6;
        }
        const LEGACY_EDITOR_HELP: &str = "Select text in Edit mode, choose a segment, then press **Link selection**. Double-click green links in Listen mode to replay them.";
        const WYSIWYG_EDITOR_HELP: &str = "Play a segment, select text, then press **Link current segment**. Double-click blue linked text to replay it.";
        for lesson in &mut self.lessons {
            lesson.markdown = lesson
                .markdown
                .replace(LEGACY_EDITOR_HELP, WYSIWYG_EDITOR_HELP);
        }
        let active_path = self
            .active()
            .map(|lesson| media_path_key(Path::new(&lesson.source_path)));
        let mut media_paths = HashSet::new();
        self.lessons
            .retain(|lesson| media_paths.insert(media_path_key(Path::new(&lesson.source_path))));
        self.active_lesson = active_path
            .and_then(|key| {
                self.lessons
                    .iter()
                    .position(|lesson| media_path_key(Path::new(&lesson.source_path)) == key)
            })
            .unwrap_or(self.active_lesson.min(self.lessons.len().saturating_sub(1)));
        self.active_lesson = self.active_lesson.min(self.lessons.len().saturating_sub(1));
        self.playback_rate = self.playback_rate.clamp(0.5, 2.0);
        self.volume = self.volume.min(100);
        let sentence_index = self.selected_sentence;
        let word_index = self.selected_word;
        if let Some(lesson) = self.active_mut() {
            for sentence in &mut lesson.sentences {
                sentence.start_ms = sentence.start_ms.min(lesson.duration_ms.saturating_sub(1));
                sentence.end_ms = sentence
                    .end_ms
                    .clamp(sentence.start_ms + 1, lesson.duration_ms.max(1));
                for word in &mut sentence.words {
                    word.start_ms = word
                        .start_ms
                        .clamp(sentence.start_ms, sentence.end_ms.saturating_sub(1));
                    word.end_ms = word.end_ms.clamp(word.start_ms + 1, sentence.end_ms);
                }
            }
            let normalized_sentence = sentence_index.min(lesson.sentences.len().saturating_sub(1));
            let normalized_word = word_index.and_then(|index| {
                lesson
                    .sentences
                    .get(normalized_sentence)
                    .and_then(|sentence| {
                        (!sentence.words.is_empty()).then_some(index.min(sentence.words.len() - 1))
                    })
            });
            self.selected_sentence = normalized_sentence;
            self.selected_word = normalized_word;
        }
    }
}

fn media_path_key(path: &Path) -> String {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let key = path.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    {
        key.to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        key.into_owned()
    }
}

pub fn format_time(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms % 3_600_000) / 60_000;
    let seconds = (ms % 60_000) / 1_000;
    let tenths = (ms % 1_000) / 100;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}.{tenths}")
    } else {
        format!("{minutes:02}:{seconds:02}.{tenths}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn word_playback_pads_both_ends_without_changing_segmentation() {
        let word = WordSegment {
            id: "w001".into(),
            label: "soon".into(),
            start_ms: 200,
            end_ms: 400,
            probability: None,
        };
        let original = serde_json::to_string(&word).unwrap();
        assert_eq!(word.playback_range(80, 1_000), (120, 480));
        assert_eq!(word.playback_range(0, 1_000), (200, 400));
        assert_eq!(word.playback_range(300, 450), (0, 450));
        assert_eq!(word.playback_range(u64::MAX, 450), (0, 450));
        assert_eq!(serde_json::to_string(&word).unwrap(), original);
    }
    #[test]
    fn formats_time() {
        assert_eq!(format_time(62_340), "01:02.3");
    }

    #[test]
    fn duplicate_media_is_reselected_instead_of_added() {
        let media = Path::new("C:/media/lesson.mp3");
        let workspace = WorkspacePaths {
            root: PathBuf::from("C:/media/lesson_bp"),
            audio: PathBuf::from("C:/media/lesson_bp/audio"),
            segmentations: PathBuf::from("C:/media/lesson_bp/segmentations"),
            dictations: PathBuf::from("C:/media/lesson_bp/dictations"),
        };
        let mut project = Project::sample(media, &workspace);
        let outcome = project.add_lesson(media, 4_000, &workspace);
        assert_eq!(outcome, AddLessonOutcome::Existing(0));
        assert_eq!(project.lessons.len(), 1);
    }

    #[test]
    fn normalize_removes_duplicates_already_saved_in_a_library() {
        let workspace = WorkspacePaths {
            root: PathBuf::from("C:/media/lesson_bp"),
            audio: PathBuf::new(),
            segmentations: PathBuf::new(),
            dictations: PathBuf::new(),
        };
        let mut project = Project::sample(Path::new("C:/media/lesson.mp3"), &workspace);
        project.lessons.push(project.lessons[0].clone());
        project.active_lesson = 1;
        project.normalize();
        assert_eq!(project.lessons.len(), 1);
        assert_eq!(project.active_lesson, 0);
    }

    #[test]
    fn old_word_alignment_is_invalidated_once_without_erasing_dictation() {
        let workspace = WorkspacePaths::for_media(
            Path::new("C:/media/lesson.mp3"),
            &crate::config::WorkspaceConfig::default(),
        );
        let mut project = Project::sample(Path::new("C:/media/lesson.mp3"), &workspace);
        project.version = 2;
        project.lessons[0].analyzed = true;
        let markdown = project.lessons[0].markdown.clone();
        project.normalize();
        assert!(!project.lessons[0].analyzed);
        assert_eq!(project.version, 6);
        assert_eq!(project.lessons[0].markdown, markdown);
        project.lessons[0].analyzed = true;
        project.normalize();
        assert!(project.lessons[0].analyzed);
    }

    #[test]
    fn legacy_loop_granularities_migrate_to_binary_loop() {
        for legacy in ["Media", "Sentence", "Word"] {
            let json = format!("\"{legacy}\"");
            assert_eq!(
                serde_json::from_str::<LoopMode>(&json).unwrap(),
                LoopMode::Loop
            );
        }
    }

    #[test]
    fn removing_active_media_selects_the_next_available_lesson() {
        let workspace = WorkspacePaths {
            root: PathBuf::from("C:/media/lesson_bp"),
            audio: PathBuf::new(),
            segmentations: PathBuf::new(),
            dictations: PathBuf::new(),
        };
        let mut project = Project::sample(Path::new("C:/media/one.mp3"), &workspace);
        project.add_lesson(Path::new("C:/media/two.mp3"), 2_000, &workspace);
        project.add_lesson(Path::new("C:/media/three.mp3"), 3_000, &workspace);
        project.select_lesson(1);

        let removed = project.remove_lesson(1).unwrap();

        assert_eq!(removed.title, "two");
        assert_eq!(project.lessons.len(), 2);
        assert_eq!(project.active_lesson, 1);
        assert_eq!(project.active().unwrap().title, "three");
    }

    #[test]
    fn removing_media_before_active_keeps_the_same_lesson_selected() {
        let workspace = WorkspacePaths {
            root: PathBuf::from("C:/media/lesson_bp"),
            audio: PathBuf::new(),
            segmentations: PathBuf::new(),
            dictations: PathBuf::new(),
        };
        let mut project = Project::sample(Path::new("C:/media/one.mp3"), &workspace);
        project.add_lesson(Path::new("C:/media/two.mp3"), 2_000, &workspace);
        project.select_lesson(1);

        project.remove_lesson(0);

        assert_eq!(project.active_lesson, 0);
        assert_eq!(project.active().unwrap().title, "two");
    }
}
