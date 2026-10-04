use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub tools: ToolConfig,
    pub workspace: WorkspaceConfig,
    pub segmentation: SegmentationConfig,
    pub playback: PlaybackConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolConfig {
    pub ffmpeg: String,
    pub ffprobe: String,
    pub whisper_model: String,
    pub vad_model: String,
    pub mpv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
    pub suffix: String,
    pub audio_directory: String,
    pub segmentation_directory: String,
    pub dictation_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SegmentationConfig {
    pub language: String,
    pub use_volume_segmentation: bool,
    pub silence_relative_db: f32,
    pub silence_threshold_db: Option<f32>,
    pub min_speech_ms: u64,
    pub whisper_workers: usize,
    pub whisper_threads: usize,
    pub use_vad: bool,
    pub silence_gap_ms: u64,
    pub vad_min_silence_ms: u64,
    pub speech_padding_ms: u64,
    pub word_boundary_padding_ms: u64,
    pub max_sentence_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackConfig {
    pub default_volume: u8,
    pub default_rate: f32,
    pub word_padding_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub locale: String,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            ffmpeg: "thirdparty/ffmpeg-n9.0-latest-win64-lgpl-shared-9.0/bin/ffmpeg.exe".into(),
            ffprobe: String::new(),
            whisper_model: "thirdparty/models/whisper/ggml-base.bin".into(),
            vad_model: "thirdparty/models/whisper-vad/ggml-silero-v6.2.0.bin".into(),
            mpv: String::new(),
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            suffix: "_bp".into(),
            audio_directory: "audio".into(),
            segmentation_directory: "segmentations".into(),
            dictation_directory: "dictations".into(),
        }
    }
}

impl Default for SegmentationConfig {
    fn default() -> Self {
        Self {
            language: "en".into(),
            use_volume_segmentation: true,
            silence_relative_db: 16.0,
            silence_threshold_db: None,
            min_speech_ms: 80,
            whisper_workers: 2,
            whisper_threads: 0,
            use_vad: true,
            silence_gap_ms: 650,
            vad_min_silence_ms: 550,
            speech_padding_ms: 200,
            word_boundary_padding_ms: 80,
            max_sentence_ms: 15_000,
        }
    }
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            default_volume: 80,
            default_rate: 1.0,
            word_padding_ms: 80,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            locale: "system".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub values: AppConfig,
    pub path: PathBuf,
    pub repo_root: PathBuf,
}

impl LoadedConfig {
    pub fn load() -> Self {
        let mut path = config_candidates()
            .into_iter()
            .find(|path| path.is_file())
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bibiparrot.toml"));
        // Installed bundles may be read-only. Keep editable preferences in the
        // user's application data directory while resources stay in the bundle.
        if env::var_os("BIBIPARROT_CONFIG").is_none() && is_packaged() {
            if let Some(directory) = eframe::storage_dir("com.bibiparrot.egui") {
                let preferences = directory.join("bibiparrot.toml");
                if preferences != path
                    && (preferences.is_file()
                        || (fs::create_dir_all(&directory).is_ok()
                            && fs::copy(&path, &preferences).is_ok()))
                {
                    path = preferences;
                }
            }
        }
        let values = fs::read_to_string(&path)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default();
        let repo_root = discover_repo_root(path.parent().unwrap_or(Path::new(".")));
        Self {
            values,
            path,
            repo_root,
        }
    }

    pub fn resolve_tool(&self, configured: &str) -> Option<PathBuf> {
        if configured.trim().is_empty() {
            return None;
        }
        let path = PathBuf::from(configured);
        let resolved = if path.is_absolute() {
            path
        } else {
            self.repo_root.join(path)
        };
        resolved.is_file().then_some(resolved)
    }

    pub fn ffmpeg(&self) -> Option<PathBuf> {
        self.resolve_tool(&self.values.tools.ffmpeg).or_else(|| {
            let name = if cfg!(windows) {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            };
            let local = env::current_exe().ok()?.parent()?.join(name);
            if local.is_file() {
                return Some(local);
            }
            env::var_os("PATH").and_then(|paths| {
                env::split_paths(&paths)
                    .map(|dir| dir.join(name))
                    .find(|path| path.is_file())
            })
        })
    }
    pub fn ffprobe(&self) -> Option<PathBuf> {
        self.resolve_tool(&self.values.tools.ffprobe)
    }
    pub fn whisper_model(&self) -> Option<PathBuf> {
        self.resolve_tool(&self.values.tools.whisper_model)
    }
    pub fn vad_model(&self) -> Option<PathBuf> {
        self.resolve_tool(&self.values.tools.vad_model)
    }
    pub fn mpv(&self) -> Option<PathBuf> {
        self.resolve_tool(&self.values.tools.mpv)
    }

    pub fn save(&self) -> Result<(), String> {
        let text = toml::to_string_pretty(&self.values)
            .map_err(|error| format!("Could not serialize settings: {error}"))?;
        fs::write(&self.path, text)
            .map_err(|error| format!("Could not save {}: {error}", self.path.display()))
    }
}

fn config_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = env::var_os("BIBIPARROT_CONFIG") {
        paths.push(PathBuf::from(path));
    }
    if is_packaged() {
        if let Some(directory) = eframe::storage_dir("com.bibiparrot.egui") {
            paths.push(directory.join("bibiparrot.toml"));
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push(parent.join("bibiparrot.toml"));
        }
    }
    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join("bibiparrot.toml"));
    }
    paths.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bibiparrot.toml"));
    paths
}

fn is_packaged() -> bool {
    env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.parent()
                .map(|parent| parent.join("thirdparty/models/whisper/ggml-base.bin"))
        })
        .is_some_and(|model| model.is_file())
}

fn discover_repo_root(start: &Path) -> PathBuf {
    for candidate in start.ancestors() {
        if candidate.join("thirdparty").is_dir() {
            return candidate.to_path_buf();
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("thirdparty").is_dir() {
                return parent.to_path_buf();
            }
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for candidate in manifest.ancestors() {
        if candidate.join("thirdparty").is_dir() {
            return candidate.to_path_buf();
        }
    }
    start.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_config_is_valid() {
        let config: AppConfig = toml::from_str(include_str!("../bibiparrot.toml")).unwrap();
        assert_eq!(
            config.tools.ffmpeg,
            "thirdparty/ffmpeg-n9.0-latest-win64-lgpl-shared-9.0/bin/ffmpeg.exe"
        );
        assert_eq!(config.segmentation.language, "en");
        assert_eq!(config.workspace.suffix, "_bp");
        assert_eq!(config.ui.locale, "system");
    }
}
