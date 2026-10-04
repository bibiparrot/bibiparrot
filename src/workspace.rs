use crate::{config::WorkspaceConfig, model::Lesson};
use serde::Serialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub audio: PathBuf,
    pub segmentations: PathBuf,
    pub dictations: PathBuf,
}

impl WorkspacePaths {
    pub fn for_media(media: &Path, config: &WorkspaceConfig) -> Self {
        let stem = sanitize(
            media
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("media"),
        );
        let parent = media.parent().unwrap_or(Path::new("."));
        let root = parent.join(format!("{stem}{}", config.suffix));
        Self {
            audio: root.join(&config.audio_directory),
            segmentations: root.join(&config.segmentation_directory),
            dictations: root.join(&config.dictation_directory),
            root,
        }
    }

    pub fn from_root(root: PathBuf, config: &WorkspaceConfig) -> Self {
        Self {
            audio: root.join(&config.audio_directory),
            segmentations: root.join(&config.segmentation_directory),
            dictations: root.join(&config.dictation_directory),
            root,
        }
    }

    pub fn create(&self) -> io::Result<()> {
        fs::create_dir_all(&self.audio)?;
        fs::create_dir_all(&self.segmentations)?;
        fs::create_dir_all(&self.dictations)?;
        Ok(())
    }
}

pub fn save_lesson(lesson: &Lesson, config: &WorkspaceConfig) -> Result<(), String> {
    let workspace = WorkspacePaths::from_root(PathBuf::from(&lesson.workspace_path), config);
    workspace.create().map_err(|error| error.to_string())?;
    fs::write(workspace.dictations.join("dictation.md"), &lesson.markdown)
        .map_err(|error| error.to_string())?;
    // Analysis owns segments.json; playback/autosave must never rewrite it.
    write_json(workspace.root.join("workspace.json"), lesson)
}

pub fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, json).map_err(|error| error.to_string())
}

fn sanitize(input: &str) -> String {
    let value: String = input
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect();
    value.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_dictation_leaves_analysis_file_untouched() {
        let config = WorkspaceConfig::default();
        let root =
            std::env::temp_dir().join(format!("bibiparrot-save-timestamps-{}", std::process::id()));
        let workspace = WorkspacePaths::from_root(root, &config);
        let project = crate::model::Project::sample(Path::new("lesson.mp3"), &workspace);
        workspace.create().unwrap();
        let path = workspace.segmentations.join("segments.json");
        let original =
            crate::analysis::segmentation_json(&project.lessons[0].sentences).unwrap() + "\n";
        fs::write(&path, &original).unwrap();
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        save_lesson(&project.lessons[0], &config).unwrap();
        let json = fs::read_to_string(workspace.segmentations.join("segments.json")).unwrap();
        assert_eq!(
            json, original,
            "Saving playback or dictation must not rewrite analysis"
        );
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), modified);
        assert!(
            !json.contains("_ms"),
            "Saving a lesson must not overwrite timestamps with milliseconds"
        );
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value[0]["words"][0]["start"], "00:00:00,230");
    }
    #[test]
    fn creates_expected_workspace_name() {
        let paths = WorkspacePaths::for_media(
            Path::new("D:/lessons/first snowfall.mp4"),
            &WorkspaceConfig::default(),
        );
        assert!(paths.root.ends_with("first_snowfall_bp"));
        assert!(paths.segmentations.ends_with("segmentations"));
    }
}
