use crate::{
    analysis::{analyze_media_with_progress, AnalysisProgress, AnalysisResult},
    config::LoadedConfig,
    localization::{install_system_fallback_fonts, LocaleManager, LOCALE_OPTIONS, SYSTEM_LOCALE},
    markdown::LinkTarget,
    markdown_editor::{MarkdownEditor, TextFormatCommand, DEFAULT_MARKDOWN_FONT_SIZE},
    media::{probe_duration, PlaybackSettings, ProcessPlayer},
    model::{format_time, AddLessonOutcome, LoopMode, MediaKind, Project},
    video::EmbeddedVideo,
    workspace::{save_lesson, WorkspacePaths},
};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use rust_i18n::t;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const SAMPLE_AUDIO: &[u8] = include_bytes!("../assets/first snowfall.mp3");
const LOGO: &[u8] = include_bytes!("../assets/bibiparrot-logo.png");
const SPONSOR: &[u8] = include_bytes!("../assets/sponsor.png");
const BIBI: &[u8] = include_bytes!("../assets/bibi-icon.png");
const TIMELINE_HEIGHT: f32 = 8.0;
const ANALYSIS_ACTIVITY_FRAMES: [&str; 4] = ["●○○○", "○●○○", "○○●○", "○○○●"];
const ANALYSIS_TIPS: [&str; 5] = [
    "Keep BibiParrot open while local Whisper is working.",
    "Long video can take several minutes to transcribe.",
    "Sentences appear first; words are added while alignment continues.",
    "Local transcription may use significant CPU or GPU resources.",
    "You can continue reading the current dictation while you wait.",
];

#[derive(Debug, PartialEq, Eq)]
struct AnalysisActivity {
    frame: &'static str,
    elapsed: String,
    tip: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaIcon {
    Open,
    Save,
    Play,
    Pause,
    Maximize,
    Restore,
    Close,
    Delete,
    Link,
    Previous,
    Next,
    Analyze,
    Workspace,
    Video,
    Left,
    Right,
    Microphone,
    Volume,
    VolumeMuted,
    Send,
    Repeat,
    Info,
    Code,
    Copy,
    Cut,
    Paste,
    Search,
}

impl FaIcon {
    #[cfg(test)]
    const ALL: [Self; 27] = [
        Self::Open,
        Self::Save,
        Self::Play,
        Self::Pause,
        Self::Maximize,
        Self::Restore,
        Self::Close,
        Self::Delete,
        Self::Link,
        Self::Previous,
        Self::Next,
        Self::Analyze,
        Self::Workspace,
        Self::Video,
        Self::Left,
        Self::Right,
        Self::Microphone,
        Self::Volume,
        Self::VolumeMuted,
        Self::Send,
        Self::Repeat,
        Self::Info,
        Self::Code,
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::Search,
    ];

    fn label(self) -> String {
        match self {
            Self::Open => t!("icons.open"),
            Self::Save => t!("icons.save"),
            Self::Play => t!("icons.play"),
            Self::Pause => t!("icons.pause"),
            Self::Maximize => t!("icons.maximize"),
            Self::Restore => t!("icons.restore"),
            Self::Close => t!("icons.close"),
            Self::Delete => t!("icons.delete"),
            Self::Link => t!("icons.link"),
            Self::Previous => t!("icons.previous"),
            Self::Next => t!("icons.next"),
            Self::Analyze => t!("icons.analyze"),
            Self::Workspace => t!("icons.workspace"),
            Self::Video => t!("icons.video"),
            Self::Left => t!("icons.collapse_left"),
            Self::Right => t!("icons.collapse_right"),
            Self::Microphone => t!("icons.microphone"),
            Self::Volume => t!("icons.volume"),
            Self::VolumeMuted => t!("icons.muted"),
            Self::Send => t!("icons.send"),
            Self::Repeat => t!("icons.repeat"),
            Self::Info => t!("icons.information"),
            Self::Code => t!("icons.markdown_source"),
            Self::Copy => t!("icons.copy"),
            Self::Cut => t!("icons.cut"),
            Self::Paste => t!("icons.paste"),
            Self::Search => t!("icons.search"),
        }
        .into_owned()
    }

    fn uri(self) -> &'static str {
        match self {
            Self::Open => "bytes://font-awesome/folder-open.svg",
            Self::Save => "bytes://font-awesome/floppy-disk.svg",
            Self::Play => "bytes://font-awesome/play.svg",
            Self::Pause => "bytes://font-awesome/pause.svg",
            Self::Maximize => "bytes://font-awesome/expand.svg",
            Self::Restore => "bytes://font-awesome/compress.svg",
            Self::Close => "bytes://font-awesome/xmark.svg",
            Self::Delete => "bytes://font-awesome/trash.svg",
            Self::Link => "bytes://font-awesome/link.svg",
            Self::Previous => "bytes://font-awesome/backward.svg",
            Self::Next => "bytes://font-awesome/forward.svg",
            Self::Analyze => "bytes://font-awesome/wand-magic-sparkles.svg",
            Self::Workspace => "bytes://font-awesome/folder.svg",
            Self::Video => "bytes://font-awesome/video.svg",
            Self::Left => "bytes://font-awesome/caret-left.svg",
            Self::Right => "bytes://font-awesome/caret-right.svg",
            Self::Microphone => "bytes://font-awesome/microphone.svg",
            Self::Volume => "bytes://font-awesome/volume-high.svg",
            Self::VolumeMuted => "bytes://font-awesome/volume-xmark.svg",
            Self::Send => "bytes://font-awesome/paper-plane.svg",
            Self::Repeat => "bytes://font-awesome/repeat.svg",
            Self::Info => "bytes://font-awesome/circle-info.svg",
            Self::Code => "bytes://font-awesome/code.svg",
            Self::Copy => "bytes://font-awesome/copy.svg",
            Self::Cut => "bytes://font-awesome/scissors.svg",
            Self::Paste => "bytes://font-awesome/paste.svg",
            Self::Search => "bytes://font-awesome/magnifying-glass.svg",
        }
    }

    fn bytes(self) -> &'static [u8] {
        macro_rules! cached_svg {
            ($path:path) => {{
                static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
                BYTES
                    .get_or_init(|| pictogram::svg!($path).to_string().into_bytes())
                    .as_slice()
            }};
        }
        match self {
            Self::Open => cached_svg!(pictogram::font_awesome::folder_open::solid),
            Self::Save => cached_svg!(pictogram::font_awesome::floppy_disk::solid),
            Self::Play => cached_svg!(pictogram::font_awesome::play::solid),
            Self::Pause => cached_svg!(pictogram::font_awesome::pause::solid),
            Self::Maximize => cached_svg!(pictogram::font_awesome::expand::solid),
            Self::Restore => cached_svg!(pictogram::font_awesome::compress::solid),
            Self::Close => cached_svg!(pictogram::font_awesome::xmark::solid),
            Self::Delete => cached_svg!(pictogram::font_awesome::trash::solid),
            Self::Link => cached_svg!(pictogram::font_awesome::link::solid),
            Self::Previous => cached_svg!(pictogram::font_awesome::backward::solid),
            Self::Next => cached_svg!(pictogram::font_awesome::forward::solid),
            Self::Analyze => cached_svg!(pictogram::font_awesome::wand_magic_sparkles::solid),
            Self::Workspace => cached_svg!(pictogram::font_awesome::folder::solid),
            Self::Video => cached_svg!(pictogram::font_awesome::video::solid),
            Self::Left => cached_svg!(pictogram::font_awesome::caret_left::solid),
            Self::Right => cached_svg!(pictogram::font_awesome::caret_right::solid),
            Self::Microphone => cached_svg!(pictogram::font_awesome::microphone::solid),
            Self::Volume => cached_svg!(pictogram::font_awesome::volume_high::solid),
            Self::VolumeMuted => cached_svg!(pictogram::font_awesome::volume_xmark::solid),
            Self::Send => cached_svg!(pictogram::font_awesome::paper_plane::solid),
            Self::Repeat => cached_svg!(pictogram::font_awesome::repeat::solid),
            Self::Info => cached_svg!(pictogram::font_awesome::circle_info::solid),
            Self::Code => cached_svg!(pictogram::font_awesome::code::solid),
            Self::Copy => cached_svg!(pictogram::font_awesome::copy::solid),
            Self::Cut => cached_svg!(pictogram::font_awesome::scissors::solid),
            Self::Paste => cached_svg!(pictogram::font_awesome::paste::solid),
            Self::Search => cached_svg!(pictogram::font_awesome::magnifying_glass::solid),
        }
    }
}

fn fa_image(icon: FaIcon, size: f32) -> egui::Image<'static> {
    egui::Image::from_bytes(icon.uri(), icon.bytes())
        .fit_to_exact_size(Vec2::splat(size))
        .alt_text(icon.label())
}

fn fa_button(icon: FaIcon) -> egui::Button<'static> {
    egui::Button::image(fa_image(icon, 13.0))
        .image_tint_follows_text_color(true)
        .min_size(Vec2::new(28.0, 24.0))
}

fn fa_text_button(icon: FaIcon, text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::image_and_text(fa_image(icon, 12.0), text).image_tint_follows_text_color(true)
}

fn fa_small_button(icon: FaIcon) -> egui::Button<'static> {
    egui::Button::image(fa_image(icon, 10.0))
        .image_tint_follows_text_color(true)
        .small()
        .min_size(Vec2::new(22.0, 18.0))
}

fn fa_player_button(icon: FaIcon) -> egui::Button<'static> {
    egui::Button::image(fa_image(icon, 12.0))
        .image_tint_follows_text_color(true)
        .min_size(Vec2::new(26.0, 22.0))
}

fn loop_mode_selector(ui: &mut egui::Ui, id_salt: &'static str, mode: &mut LoopMode, width: f32) {
    ui.add(fa_image(FaIcon::Repeat, 12.0))
        .on_hover_text(t!("player.loop_mode"));
    let response = egui::ComboBox::from_id_salt(id_salt)
        .width(width)
        .selected_text(loop_mode_label(*mode))
        .show_ui(ui, |ui| {
            for candidate in [LoopMode::Off, LoopMode::Loop] {
                ui.selectable_value(mode, candidate, loop_mode_label(candidate));
            }
        });
    response.response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::ComboBox,
            ui.is_enabled(),
            t!("player.loop_mode"),
        )
    });
}

fn loop_mode_label(mode: LoopMode) -> std::borrow::Cow<'static, str> {
    match mode {
        LoopMode::Off => t!("player.not_loop"),
        LoopMode::Loop => t!("player.loop"),
    }
}

fn volume_button(ui: &mut egui::Ui, volume: u8) -> egui::Response {
    let (icon, tooltip) = if volume == 0 {
        (FaIcon::VolumeMuted, t!("player.unmute"))
    } else {
        (FaIcon::Volume, t!("player.mute"))
    };
    ui.add_sized([26.0, 22.0], fa_player_button(icon))
        .on_hover_text(tooltip)
}

fn source_mode_button(ui: &mut egui::Ui, source_mode: bool) -> egui::Response {
    ui.add(fa_button(FaIcon::Code).selected(source_mode))
        .on_hover_text(if source_mode {
            t!("dictation.show_wysiwyg")
        } else {
            t!("dictation.show_source")
        })
}

fn format_w_button(ui: &mut egui::Ui, label: impl Into<String>, text: RichText) -> egui::Response {
    let label = label.into();
    let response = ui
        .add(egui::Button::new(text).min_size(Vec2::new(28.0, 24.0)))
        .on_hover_text(&label);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label.clone())
    });
    response
}

fn font_size_selector(ui: &mut egui::Ui, font_size: &mut u16) -> bool {
    let mut selected = false;
    egui::ComboBox::from_id_salt("dictation-font-size")
        .width(52.0)
        .selected_text(format!("{} px", *font_size))
        .show_ui(ui, |ui| {
            for size in [12, 14, 16, 18, 24, 32] {
                selected |= ui
                    .selectable_value(font_size, size, format!("{size} px"))
                    .clicked();
            }
        })
        .response
        .on_hover_text(t!("dictation.font_size"));
    selected
}

fn paragraph_style_selector(ui: &mut egui::Ui, heading_level: &mut usize) -> bool {
    let mut selected_style = false;
    let selected = if *heading_level == 0 {
        t!("dictation.normal_text").into_owned()
    } else {
        format!("H{}", *heading_level)
    };
    egui::ComboBox::from_id_salt("dictation-paragraph-style")
        .width(74.0)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            selected_style |= ui
                .selectable_value(heading_level, 0, t!("dictation.normal_text"))
                .clicked();
            for level in 1..=6 {
                selected_style |= ui
                    .selectable_value(heading_level, level, format!("H{level}"))
                    .clicked();
            }
        })
        .response
        .on_hover_text(t!("dictation.paragraph_style"));
    selected_style
}

fn toggle_muted_volume(volume: &mut u8, volume_before_mute: &mut u8) {
    if *volume == 0 {
        *volume = (*volume_before_mute).max(1);
    } else {
        *volume_before_mute = *volume;
        *volume = 0;
    }
}

fn bottom_aligned_control_bar(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), add_contents);
}

#[derive(Default)]
struct PlayerControlActions {
    back: bool,
    toggle_play: bool,
    forward: bool,
    toggle_mute: bool,
    volume_changed: bool,
    rate_changed: bool,
}

fn player_control_strip(
    ui: &mut egui::Ui,
    is_playing: bool,
    volume: &mut u8,
    playback_rate: &mut f32,
    loop_mode: &mut LoopMode,
) -> PlayerControlActions {
    let available_width = ui.available_width();
    let show_volume_slider = available_width >= 400.0;
    let show_speed = available_width >= 300.0;
    let narrow = available_width < 520.0;
    let mut actions = PlayerControlActions::default();

    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 22.0;
        ui.spacing_mut().button_padding = Vec2::new(4.0, 1.5);
        ui.spacing_mut().item_spacing.x = if narrow { 3.0 } else { 5.0 };
        ui.horizontal(|ui| {
            if ui
                .add_sized([26.0, 22.0], fa_player_button(FaIcon::Previous))
                .on_hover_text(t!("player.back_3"))
                .clicked()
            {
                actions.back = true;
            }
            if ui
                .add_sized(
                    [26.0, 22.0],
                    fa_player_button(if is_playing {
                        FaIcon::Pause
                    } else {
                        FaIcon::Play
                    }),
                )
                .on_hover_text(if is_playing {
                    t!("icons.pause")
                } else {
                    t!("icons.play")
                })
                .clicked()
            {
                actions.toggle_play = true;
            }
            if ui
                .add_sized([26.0, 22.0], fa_player_button(FaIcon::Next))
                .on_hover_text(t!("player.forward_3"))
                .clicked()
            {
                actions.forward = true;
            }
            if !narrow {
                ui.separator();
            }
            if volume_button(ui, *volume).clicked() {
                actions.toggle_mute = true;
            }
            if show_volume_slider {
                let response = ui.add_sized(
                    [if narrow { 72.0 } else { 96.0 }, 22.0],
                    egui::Slider::new(volume, 0..=100).suffix("%"),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::slider(
                        ui.is_enabled(),
                        f64::from(*volume),
                        t!("player.volume_level"),
                    )
                });
                if response.changed() {
                    actions.volume_changed = true;
                }
            }
            if show_speed {
                let response = egui::ComboBox::from_id_salt("player-speed")
                    .width(if narrow { 54.0 } else { 60.0 })
                    .selected_text(format!("{playback_rate:.2}×"))
                    .show_ui(ui, |ui| {
                        for speed in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0] {
                            ui.selectable_value(playback_rate, speed, format!("{speed:.2}×"));
                        }
                    });
                actions.rate_changed = response.response.changed();
                response.response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::ComboBox,
                        ui.is_enabled(),
                        t!("player.speed"),
                    )
                });
            }
            loop_mode_selector(
                ui,
                "player-loop",
                loop_mode,
                if narrow { 72.0 } else { 84.0 },
            );
        });
    });

    actions
}

#[derive(Clone, Copy)]
struct TimelinePalette {
    background: Color32,
    fill: Color32,
    selected: Color32,
}

fn analysis_activity(elapsed: Duration) -> AnalysisActivity {
    let elapsed_millis = elapsed.as_millis();
    let frame_index = (elapsed_millis / 250) as usize % ANALYSIS_ACTIVITY_FRAMES.len();
    let tip_index = (elapsed.as_secs() / 4) as usize % ANALYSIS_TIPS.len();
    let total_seconds = elapsed.as_secs();
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    let elapsed = if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    };
    AnalysisActivity {
        frame: ANALYSIS_ACTIVITY_FRAMES[frame_index],
        elapsed,
        tip: ANALYSIS_TIPS[tip_index],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockTab {
    Player,
    MediaLibrary,
    Segmentation,
    Dictation,
    Assistant,
}

impl DockTab {
    const ALL: [Self; 5] = [
        Self::Player,
        Self::MediaLibrary,
        Self::Segmentation,
        Self::Dictation,
        Self::Assistant,
    ];
    fn title(self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Player => t!("dock.player"),
            Self::MediaLibrary => t!("dock.media_library"),
            Self::Segmentation => t!("dock.segmentation"),
            Self::Dictation => t!("dock.dictation"),
            Self::Assistant => t!("dock.assistant"),
        }
    }
}

#[derive(Debug, Clone)]
struct ChatMessage {
    assistant: bool,
    text: String,
}

#[derive(Debug)]
enum AnalysisMessage {
    Progress(AnalysisProgress),
    Complete(AnalysisResult),
}

pub struct BibiParrotApp {
    project: Project,
    config: LoadedConfig,
    locale_manager: LocaleManager,
    player: ProcessPlayer,
    embedded_video: EmbeddedVideo,
    dock_state: DockState<DockTab>,
    maximized: Option<DockTab>,
    logo: egui::TextureHandle,
    sponsor: egui::TextureHandle,
    bibi: egui::TextureHandle,
    dictation_editor: MarkdownEditor,
    dictation_font_size: u16,
    dictation_heading_level: usize,
    show_dictation_find_replace: bool,
    dictation_find: String,
    dictation_replace: String,
    current_play_target: Option<LinkTarget>,
    show_video: bool,
    media_scrub_position: Option<u64>,
    volume_before_mute: u8,
    show_about: bool,
    status: String,
    dirty: bool,
    project_path: PathBuf,
    analysis_receiver: Option<Receiver<Result<AnalysisMessage, String>>>,
    analysis_running: bool,
    analysis_sentences_ready: bool,
    analysis_started_at: Option<Instant>,
    analysis_lesson_index: Option<usize>,
    analysis_error: Option<(usize, String)>,
    assistant_input: String,
    chat: Vec<ChatMessage>,
    qa_screenshot_path: Option<PathBuf>,
    qa_frame_count: u8,
    qa_requested: bool,
    qa_autoplay: bool,
    qa_autoplay_started: bool,
    show_media_library: bool,
    show_segmentation: bool,
    show_assistant: bool,
    show_segmentation_tips: bool,
    pending_collapse: Option<DockTab>,
    pending_dock_rebuild: bool,
}

impl BibiParrotApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_theme(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        install_system_fallback_fonts(&cc.egui_ctx);
        let logo = load_texture(&cc.egui_ctx, "bibiparrot-logo", LOGO);
        let sponsor = load_texture(&cc.egui_ctx, "bibiparrot-sponsor", SPONSOR);
        let bibi = load_texture(&cc.egui_ctx, "bibi-assistant", BIBI);
        let config = LoadedConfig::load();
        let locale_manager = LocaleManager::new(&config.values.ui.locale);
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            t!("window.title").into_owned(),
        ));
        let qa_data_dir = std::env::var_os("BIBIPARROT_QA_DATA_DIR")
            .filter(|_| std::env::var_os("BIBIPARROT_QA_SCREENSHOT").is_some())
            .map(PathBuf::from);
        let data_dir = qa_data_dir
            .or_else(|| eframe::storage_dir("com.bibiparrot.egui"))
            .unwrap_or_else(|| std::env::temp_dir().join("bibiparrot-egui"));
        let _ = fs::create_dir_all(&data_dir);
        let sample_path = data_dir.join("first_snowfall.mp3");
        if !sample_path.exists() {
            let _ = fs::write(&sample_path, SAMPLE_AUDIO);
        }
        let sample_workspace = WorkspacePaths::for_media(&sample_path, &config.values.workspace);
        let _ = sample_workspace.create();
        let project_path = data_dir.join("library-v2.json");
        let mut project = fs::read_to_string(&project_path)
            .ok()
            .and_then(|json| serde_json::from_str::<Project>(&json).ok())
            .unwrap_or_else(|| Project::sample(&sample_path, &sample_workspace));
        project.normalize();
        let qa_screenshot_path = std::env::var_os("BIBIPARROT_QA_SCREENSHOT").map(PathBuf::from);
        if qa_screenshot_path.is_some() && !project.lessons.is_empty() {
            let qa_lesson = std::env::var("BIBIPARROT_QA_LESSON")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0)
                .min(project.lessons.len() - 1);
            project.select_lesson(qa_lesson);
        }
        if project.volume == 0 {
            project.volume = config.values.playback.default_volume;
        }
        let player = ProcessPlayer::new(&config);
        let status = t!(
            "status.ready",
            backend = player.backend_name(),
            path = config.path.display().to_string()
        )
        .into_owned();
        let show_video = project
            .active()
            .is_some_and(|lesson| lesson.kind == MediaKind::Video);
        let initial_kind = if show_video {
            MediaKind::Video
        } else {
            MediaKind::Audio
        };
        let qa_collapsed_docks = qa_screenshot_path.is_some()
            && std::env::var_os("BIBIPARROT_QA_COLLAPSED_DOCKS").is_some();
        let show_media_library = !qa_collapsed_docks;
        let show_segmentation = !qa_collapsed_docks;
        let show_assistant = !qa_collapsed_docks;
        let dock_state = dock_state_for_kind(
            show_media_library,
            show_segmentation,
            show_assistant,
            initial_kind,
        );
        let volume_before_mute = project.volume.max(1);
        let auto_analyze = (qa_screenshot_path.is_none()
            && project.active().is_some_and(|lesson| !lesson.analyzed))
            || (qa_screenshot_path.is_some()
                && std::env::var_os("BIBIPARROT_QA_ANALYZE").is_some());
        let mut app = Self {
            project,
            config,
            locale_manager,
            player,
            embedded_video: EmbeddedVideo::new(),
            dock_state,
            maximized: None,
            logo,
            sponsor,
            bibi,
            dictation_editor: MarkdownEditor::default(),
            dictation_font_size: DEFAULT_MARKDOWN_FONT_SIZE,
            dictation_heading_level: 0,
            show_dictation_find_replace: false,
            dictation_find: String::new(),
            dictation_replace: String::new(),
            current_play_target: None,
            show_video,
            media_scrub_position: None,
            volume_before_mute,
            show_about: false,
            status,
            dirty: false,
            project_path,
            analysis_receiver: None,
            analysis_running: false,
            analysis_sentences_ready: false,
            analysis_started_at: None,
            assistant_input: String::new(),
            analysis_lesson_index: None,
            analysis_error: None,
            chat: vec![ChatMessage {
                assistant: true,
                text: t!("assistant.welcome").into_owned(),
            }],
            qa_screenshot_path,
            qa_frame_count: 0,
            qa_requested: false,
            qa_autoplay: std::env::var_os("BIBIPARROT_QA_AUTOPLAY").is_some(),
            qa_autoplay_started: false,
            show_media_library,
            show_segmentation,
            show_assistant,
            show_segmentation_tips: false,
            pending_collapse: None,
            pending_dock_rebuild: false,
        };
        if auto_analyze {
            app.start_analysis();
        }
        app
    }

    fn open_media_dialog(&mut self) {
        if let Some(paths) = rfd::FileDialog::new()
            .add_filter(
                "Audio and video",
                &[
                    "mp3", "wav", "flac", "ogg", "m4a", "aac", "mp4", "mkv", "webm", "mov", "avi",
                ],
            )
            .pick_files()
        {
            self.add_files(paths);
        }
    }

    fn add_files(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if let Some(index) = self.project.lesson_index_for_path(&path) {
                self.project.select_lesson(index);
                self.status = format!("Already in Media Library · {}", path.display());
                continue;
            }
            let workspace = WorkspacePaths::for_media(&path, &self.config.values.workspace);
            match workspace.create() {
                Ok(()) => {
                    let duration = probe_duration(&path, &self.config).unwrap_or(60_000);
                    match self.project.add_lesson(&path, duration, &workspace) {
                        AddLessonOutcome::Added(_) => {
                            self.status = format!(
                                "Opened {} · workspace {}",
                                path.display(),
                                workspace.root.display()
                            );
                            self.dirty = true;
                        }
                        AddLessonOutcome::Existing(_) => {
                            self.status = format!("Already in Media Library · {}", path.display());
                        }
                    }
                    self.current_play_target = None;
                }
                Err(error) => {
                    self.status =
                        format!("Could not create workspace for {}: {error}", path.display())
                }
            }
        }
        self.player.stop();
        self.embedded_video.reset();
        self.show_video = self
            .project
            .active()
            .is_some_and(|lesson| lesson.kind == MediaKind::Video);
        self.media_scrub_position = None;
        self.pending_dock_rebuild = true;
        if self.project.active().is_some_and(|lesson| !lesson.analyzed) {
            self.start_analysis();
        }
    }

    fn remove_media_from_library(&mut self, index: usize) {
        let removed_active = index == self.project.active_lesson;
        let Some(lesson) = self.project.remove_lesson(index) else {
            return;
        };

        if removed_active {
            self.player.stop();
            self.embedded_video.reset();
            self.current_play_target = None;
            self.media_scrub_position = None;
            self.show_video = self
                .project
                .active()
                .is_some_and(|active| active.kind == MediaKind::Video);
            self.pending_dock_rebuild = true;
        }
        self.analysis_error = None;
        self.dirty = true;
        self.save_project();
        if !self.dirty {
            self.status = format!(
                "Removed {} from Media Library · source and workspace kept",
                lesson.title
            );
        }
    }

    fn save_project(&mut self) {
        self.project.normalize();
        let result = serde_json::to_string_pretty(&self.project)
            .map_err(|error| error.to_string())
            .and_then(|json| fs::write(&self.project_path, json).map_err(|error| error.to_string()))
            .and_then(|()| {
                if let Some(lesson) = self.project.active() {
                    save_lesson(lesson, &self.config.values.workspace)
                } else {
                    Ok(())
                }
            });
        match result {
            Ok(()) => {
                self.dirty = false;
                self.status = "Library and dictation saved".into();
            }
            Err(error) => self.status = format!("Save failed: {error}"),
        }
    }

    fn save_dictation_as(&mut self) {
        let Some(lesson) = self.project.active() else {
            return;
        };
        let default = PathBuf::from(&lesson.workspace_path)
            .join(&self.config.values.workspace.dictation_directory)
            .join("dictation.md");
        let picked = rfd::FileDialog::new()
            .set_file_name("dictation.md")
            .set_directory(default.parent().unwrap_or(Path::new(".")))
            .save_file();
        if let Some(path) = picked {
            match fs::write(&path, &lesson.markdown) {
                Ok(()) => self.status = format!("Dictation saved to {}", path.display()),
                Err(error) => self.status = format!("Could not save dictation: {error}"),
            }
        }
    }

    fn start_analysis(&mut self) {
        if self.analysis_running {
            self.status = "Whisper analysis is already running".into();
            return;
        }
        let Some(lesson) = self.project.active() else {
            self.status = "Open media first".into();
            return;
        };
        let source = PathBuf::from(&lesson.source_path);
        let workspace = WorkspacePaths::from_root(
            PathBuf::from(&lesson.workspace_path),
            &self.config.values.workspace,
        );
        let config = self.config.clone();
        let lesson_index = self.project.active_lesson;
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = analyze_media_with_progress(
                lesson_index,
                &source,
                &workspace,
                &config,
                |progress| {
                    let _ = sender.send(Ok(AnalysisMessage::Progress(progress)));
                },
            );
            let _ = sender.send(result.map(AnalysisMessage::Complete));
        });
        self.analysis_receiver = Some(receiver);
        self.analysis_running = true;
        self.analysis_sentences_ready = false;
        self.analysis_started_at = Some(Instant::now());
        self.analysis_lesson_index = Some(lesson_index);
        self.analysis_error = None;
        self.status = format!("Extracting audio and running Whisper for {}…", lesson.title);
    }

    fn poll_analysis(&mut self) {
        let Some(receiver) = &self.analysis_receiver else {
            return;
        };
        match receiver.try_recv() {
            Ok(Ok(AnalysisMessage::Progress(AnalysisProgress::Sentences {
                lesson_index,
                sentences,
            }))) => {
                let count = sentences.len();
                if let Some(lesson) = self.project.lessons.get_mut(lesson_index) {
                    lesson.sentences = sentences;
                }
                if self.project.active_lesson == lesson_index {
                    self.project.selected_sentence = 0;
                    self.project.selected_word = None;
                }
                self.analysis_sentences_ready = true;
                self.status = format!(
                    "Created {count} audio segments · recognizing words in the background…"
                );
            }
            Ok(Ok(AnalysisMessage::Progress(AnalysisProgress::SentenceWords {
                lesson_index,
                sentence_index,
                sentence,
            }))) => {
                if let Some(existing) = self
                    .project
                    .lessons
                    .get_mut(lesson_index)
                    .and_then(|lesson| lesson.sentences.get_mut(sentence_index))
                {
                    *existing = sentence;
                }
                self.status = format!("Recognition: sentence {} is ready", sentence_index + 1);
            }
            Ok(Ok(AnalysisMessage::Complete(result))) => {
                let count = result.sentences.len();
                let words = result
                    .sentences
                    .iter()
                    .map(|sentence| sentence.words.len())
                    .sum::<usize>();
                let raw_json = result.raw_json.clone();
                self.project.apply_analysis(
                    result.lesson_index,
                    result.extracted_audio,
                    result.sentences,
                );
                self.analysis_running = false;
                self.analysis_sentences_ready = false;
                self.analysis_started_at = None;
                self.analysis_lesson_index = None;
                self.analysis_receiver = None;
                self.dirty = true;
                self.status = format!(
                    "Whisper created {count} sentences and {words} words · {}",
                    raw_json.display()
                );
                self.save_project();
            }
            Ok(Err(error)) => {
                let lesson_index = self
                    .analysis_lesson_index
                    .unwrap_or(self.project.active_lesson);
                self.analysis_running = false;
                self.analysis_sentences_ready = false;
                self.analysis_started_at = None;
                self.analysis_lesson_index = None;
                self.analysis_receiver = None;
                self.analysis_error = Some((lesson_index, error.clone()));
                self.status = format!("Segmentation failed: {error}");
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.analysis_running = false;
                self.analysis_sentences_ready = false;
                self.analysis_started_at = None;
                let lesson_index = self
                    .analysis_lesson_index
                    .unwrap_or(self.project.active_lesson);
                self.analysis_lesson_index = None;
                self.analysis_receiver = None;
                let error = "Whisper worker stopped unexpectedly".to_string();
                self.analysis_error = Some((lesson_index, error.clone()));
                self.status = error;
            }
        }
    }

    fn resolve_target(&self, target: &LinkTarget) -> Option<(usize, Option<usize>, u64, u64)> {
        let lesson = self.project.active()?;
        match target {
            LinkTarget::Sentence { sentence_id } => {
                let sentence_index = lesson
                    .sentences
                    .iter()
                    .position(|sentence| &sentence.id == sentence_id)?;
                let sentence = &lesson.sentences[sentence_index];
                Some((sentence_index, None, sentence.start_ms, sentence.end_ms))
            }
            LinkTarget::Word {
                sentence_id,
                word_id,
            } => {
                let sentence_index = lesson
                    .sentences
                    .iter()
                    .position(|sentence| &sentence.id == sentence_id)?;
                let sentence = &lesson.sentences[sentence_index];
                let word_index = sentence.words.iter().position(|word| &word.id == word_id)?;
                let word = &sentence.words[word_index];
                let (start, end) = word.playback_range(
                    self.config.values.playback.word_padding_ms,
                    lesson.duration_ms,
                );
                Some((sentence_index, Some(word_index), start, end))
            }
        }
    }

    fn play_target(&mut self, target: LinkTarget) {
        let Some((sentence_index, word_index, start_ms, end_ms)) = self.resolve_target(&target)
        else {
            self.status = "That link no longer matches the selected media segmentation".into();
            return;
        };
        self.project.selected_sentence = sentence_index;
        self.project.selected_word = word_index.or_else(|| {
            self.project
                .selected_sentence()
                .and_then(|sentence| (!sentence.words.is_empty()).then_some(0))
        });
        self.current_play_target = Some(target);
        self.play_range(start_ms, end_ms);
    }

    fn current_range(&self) -> Option<(u64, u64)> {
        let lesson = self.project.active()?;
        if let Some(target) = &self.current_play_target {
            self.resolve_target(target)
                .map(|(_, _, start_ms, end_ms)| (start_ms, end_ms))
        } else {
            Some((0, lesson.duration_ms))
        }
    }

    fn play_range(&mut self, start_ms: u64, end_ms: u64) {
        let Some(lesson) = self.project.active() else {
            return;
        };
        let path = PathBuf::from(&lesson.playback_path);
        let kind = lesson.kind;
        let looping = self.project.loop_mode == LoopMode::Loop;
        match self.player.start(
            &path,
            kind,
            start_ms,
            end_ms,
            PlaybackSettings {
                looping,
                rate: self.project.playback_rate,
                volume: self.project.volume,
            },
        ) {
            Ok(()) => {
                self.status = format!(
                    "Playing {}–{} · {}",
                    format_time(start_ms),
                    format_time(end_ms),
                    loop_mode_label(self.project.loop_mode)
                )
            }
            Err(error) => self.status = error,
        }
    }

    fn play_media_from(&mut self, start_ms: u64) {
        let Some(lesson) = self.project.active() else {
            return;
        };
        let path = PathBuf::from(&lesson.playback_path);
        let kind = lesson.kind;
        let duration_ms = lesson.duration_ms.max(1);
        let start_ms = start_ms.min(duration_ms.saturating_sub(1));
        self.current_play_target = None;
        match self.player.start(
            &path,
            kind,
            start_ms,
            duration_ms,
            PlaybackSettings {
                looping: self.project.loop_mode == LoopMode::Loop,
                rate: self.project.playback_rate,
                volume: self.project.volume,
            },
        ) {
            Ok(()) => {
                self.status = format!("Playing from {}", format_time(start_ms));
            }
            Err(error) => self.status = error,
        }
    }

    fn set_loop_mode(&mut self, mode: LoopMode) {
        if self.project.loop_mode == mode {
            return;
        }
        self.project.loop_mode = mode;
        self.player.set_looping(mode == LoopMode::Loop);
        self.status = if mode == LoopMode::Loop {
            "Loop enabled for the current playback range".into()
        } else {
            "Not loop · current playback will stop at the range end".into()
        };
        self.dirty = true;
    }

    fn set_audio_volume(&mut self, volume: u8) {
        let volume = volume.min(100);
        if volume == 0 && self.project.volume > 0 {
            self.volume_before_mute = self.project.volume;
        } else if volume > 0 {
            self.volume_before_mute = volume;
        }
        self.project.volume = volume;
        match self.player.set_volume(volume) {
            Ok(()) => {
                self.status = if volume == 0 {
                    t!("status.audio_muted").into_owned()
                } else {
                    t!("status.volume", volume = volume).into_owned()
                };
            }
            Err(error) => self.status = error,
        }
        self.dirty = true;
    }

    fn toggle_audio_mute(&mut self) {
        let mut volume = self.project.volume;
        toggle_muted_volume(&mut volume, &mut self.volume_before_mute);
        self.set_audio_volume(volume);
    }

    fn toggle_play(&mut self) {
        if self.player.is_playing() {
            self.player.pause();
            self.status = format!("Paused at {}", format_time(self.player.position_ms()));
        } else if self.player.position_ms() > self.player.range().0
            && self.player.position_ms() < self.player.range().1
        {
            match self.player.resume() {
                Ok(()) => self.status = "Playback resumed".into(),
                Err(error) => self.status = error,
            }
        } else if let Some((start, end)) = self.current_range() {
            self.play_range(start, end);
        }
    }

    fn play_relative_sentence(&mut self, delta: isize) {
        let Some(lesson) = self.project.active() else {
            return;
        };
        if lesson.sentences.is_empty() {
            return;
        }
        let next = (self.project.selected_sentence as isize + delta)
            .rem_euclid(lesson.sentences.len() as isize) as usize;
        let id = lesson.sentences[next].id.clone();
        self.play_target(LinkTarget::Sentence { sentence_id: id });
    }

    fn link_selection(&mut self) {
        let Some(target) = self.current_play_target.clone() else {
            self.status = "Select and play a sentence or word segment first".into();
            return;
        };
        match self.dictation_editor.link_selection(target) {
            Ok(markdown) => {
                if let Some(lesson) = self.project.active_mut() {
                    lesson.markdown = markdown;
                    self.dirty = true;
                    self.status =
                        "Selection linked in blue. Double-click it to replay the segment.".into();
                }
            }
            Err(message) => self.status = message.into(),
        }
    }

    fn select_locale(&mut self, ctx: &egui::Context, selection: &str) {
        self.locale_manager.select(selection);
        self.config.values.ui.locale = self.locale_manager.selection().to_owned();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            t!("window.title").into_owned(),
        ));
        self.status = match self.config.save() {
            Ok(()) => t!(
                "status.language_changed",
                language = self.locale_manager.active_native_name()
            )
            .into_owned(),
            Err(error) => error,
        };
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu-bar")
            .exact_height(27.0)
            .show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button(t!("menu.file"), |ui| {
                        if ui
                            .add(fa_text_button(FaIcon::Open, t!("actions.open_media")))
                            .clicked()
                        {
                            ui.close();
                            self.open_media_dialog();
                        }
                        if ui
                            .add(fa_text_button(
                                FaIcon::Save,
                                t!("actions.save_dictation_as"),
                            ))
                            .clicked()
                        {
                            ui.close();
                            self.save_dictation_as();
                        }
                        if ui
                            .add(fa_text_button(FaIcon::Save, t!("actions.save_workspace")))
                            .clicked()
                        {
                            ui.close();
                            self.save_project();
                        }
                        ui.separator();
                        if ui
                            .add(fa_text_button(FaIcon::Close, t!("actions.quit")))
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button(t!("menu.view"), |ui| {
                        if ui
                            .add(fa_text_button(
                                FaIcon::Restore,
                                t!("actions.restore_default_layout"),
                            ))
                            .clicked()
                        {
                            self.show_media_library = true;
                            self.show_segmentation = true;
                            self.show_assistant = true;
                            self.rebuild_dock_state();
                            self.maximized = None;
                            ui.close();
                        }
                        ui.separator();
                        for tab in DockTab::ALL {
                            if ui
                                .add(
                                    egui::Button::image_and_text(
                                        fa_image(FaIcon::Maximize, 12.0),
                                        t!("actions.maximize_dock", dock = tab.title())
                                            .into_owned(),
                                    )
                                    .image_tint_follows_text_color(true),
                                )
                                .clicked()
                            {
                                self.maximized = Some(tab);
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button(t!("menu.tools"), |ui| {
                        if ui
                            .add(fa_text_button(
                                FaIcon::Analyze,
                                t!("actions.intelligent_segmentation"),
                            ))
                            .clicked()
                        {
                            ui.close();
                            self.start_analysis();
                        }
                        if ui
                            .add(fa_text_button(
                                FaIcon::Workspace,
                                t!("actions.open_workspace"),
                            ))
                            .clicked()
                        {
                            ui.close();
                            self.open_current_workspace();
                        }
                        if ui
                            .add(fa_text_button(FaIcon::Open, t!("actions.open_config")))
                            .clicked()
                        {
                            ui.close();
                            open_in_file_manager(
                                self.config.path.parent().unwrap_or(Path::new(".")),
                            );
                        }
                    });
                    ui.menu_button(t!("menu.language"), |ui| {
                        let selection = self.locale_manager.selection();
                        let system_label = format!(
                            "{} · {} → {}",
                            t!("common.system_locale"),
                            self.locale_manager.system_locale(),
                            self.locale_manager.active_native_name()
                        );
                        for option in LOCALE_OPTIONS {
                            let label = if option.code == SYSTEM_LOCALE {
                                system_label.clone()
                            } else {
                                option.native_name.to_owned()
                            };
                            if ui
                                .selectable_label(selection == option.code, label)
                                .clicked()
                            {
                                self.select_locale(ctx, option.code);
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button(t!("menu.help"), |ui| {
                        if ui
                            .add(fa_text_button(FaIcon::Info, t!("actions.about")))
                            .clicked()
                        {
                            self.show_about = true;
                            ui.close();
                        }
                    });
                    ui.separator();
                    ui.label(
                        RichText::new(t!("actions.drag_docks"))
                            .size(10.0)
                            .color(Color32::from_rgb(91, 112, 105)),
                    );
                });
            });
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("main-toolbar")
            .exact_height(38.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(244, 249, 247))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(207, 222, 216)))
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add(
                        egui::Image::new((self.logo.id(), Vec2::new(132.0, 22.0))).corner_radius(4),
                    );
                    ui.separator();
                    if ui
                        .add(fa_button(FaIcon::Open))
                        .on_hover_text(t!("actions.open_media"))
                        .clicked()
                    {
                        self.open_media_dialog();
                    }
                    if ui
                        .add(fa_button(FaIcon::Save))
                        .on_hover_text(t!("actions.save_project"))
                        .clicked()
                    {
                        self.save_project();
                    }
                    let analyze = ui
                        .add_enabled(!self.analysis_running, fa_button(FaIcon::Analyze))
                        .on_hover_text(if self.analysis_running {
                            t!("actions.analysis_running")
                        } else {
                            t!("actions.intelligent_segmentation")
                        });
                    if analyze.clicked() {
                        self.start_analysis();
                    }
                    ui.separator();
                    let (play_icon, play_tip) = if self.player.is_playing() {
                        (FaIcon::Pause, t!("icons.pause"))
                    } else {
                        (FaIcon::Play, t!("icons.play"))
                    };
                    if ui
                        .add(fa_button(play_icon))
                        .on_hover_text(play_tip)
                        .clicked()
                    {
                        self.toggle_play();
                    }
                    if ui
                        .add(fa_button(FaIcon::Previous))
                        .on_hover_text(t!("actions.previous_sentence"))
                        .clicked()
                    {
                        self.play_relative_sentence(-1);
                    }
                    if ui
                        .add(fa_button(FaIcon::Next))
                        .on_hover_text(t!("actions.next_sentence"))
                        .clicked()
                    {
                        self.play_relative_sentence(1);
                    }
                    ui.separator();
                    let mut loop_mode = self.project.loop_mode;
                    loop_mode_selector(ui, "toolbar-loop", &mut loop_mode, 88.0);
                    self.set_loop_mode(loop_mode);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let color = if self.dirty {
                            Color32::from_rgb(166, 100, 18)
                        } else {
                            Color32::from_rgb(24, 132, 78)
                        };
                        ui.label(
                            RichText::new(if self.dirty {
                                t!("common.unsaved")
                            } else {
                                t!("common.saved")
                            })
                            .strong()
                            .size(9.0)
                            .color(color),
                        );
                    });
                });
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status-bar")
            .exact_height(24.0)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(240, 247, 244))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(207, 222, 216)))
                    .inner_margin(egui::Margin::symmetric(10, 3)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.status)
                            .size(9.0)
                            .color(Color32::from_rgb(73, 94, 87)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} · {} media",
                                self.player.backend_name(),
                                self.project.lessons.len()
                            ))
                            .size(9.0)
                            .color(Color32::from_rgb(22, 132, 76)),
                        );
                    });
                });
            });
    }

    fn show_docks(&mut self, ctx: &egui::Context) {
        if let Some(tab) = self.maximized {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(tab.title());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(fa_button(FaIcon::Restore))
                            .on_hover_text(t!("actions.restore_layout"))
                            .clicked()
                        {
                            self.maximized = None;
                        }
                    });
                });
                ui.separator();
                self.render_tab(ui, tab);
            });
            return;
        }
        self.show_collapsed_dock_rails(ctx);
        let mut dock_state = std::mem::replace(&mut self.dock_state, DockState::new(Vec::new()));
        let style = Style::from_egui(ctx.style().as_ref());
        DockArea::new(&mut dock_state)
            .style(style)
            .show_close_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .show(ctx, self);
        self.dock_state = dock_state;
        if let Some(tab) = self.pending_collapse.take() {
            match tab {
                DockTab::MediaLibrary => self.show_media_library = false,
                DockTab::Segmentation => self.show_segmentation = false,
                DockTab::Assistant => self.show_assistant = false,
                DockTab::Player | DockTab::Dictation => {}
            }
            self.rebuild_dock_state();
        }
        if self.pending_dock_rebuild {
            self.pending_dock_rebuild = false;
            self.rebuild_dock_state();
        }
    }

    fn show_collapsed_dock_rails(&mut self, ctx: &egui::Context) {
        let available = ctx.available_rect();
        let control_top = collapsed_dock_control_top(available, self.dock_media_kind());
        let mut restored = false;
        if !self.show_media_library || !self.show_segmentation {
            restored |= egui::Area::new(egui::Id::new("collapsed-left-dock-controls"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(available.left() + 2.0, control_top))
                .movable(false)
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    collapsed_left_dock_buttons(
                        ui,
                        &mut self.show_media_library,
                        &mut self.show_segmentation,
                    )
                })
                .inner;
        }
        if !self.show_assistant {
            restored |= egui::Area::new(egui::Id::new("collapsed-right-dock-controls"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(available.right() - 24.0, control_top))
                .movable(false)
                .show(ctx, |ui| {
                    collapsed_right_dock_buttons(ui, &mut self.show_assistant)
                })
                .inner;
        }
        if restored {
            self.rebuild_dock_state();
        }
    }

    fn dock_media_kind(&self) -> MediaKind {
        if self.show_video {
            self.project
                .active()
                .map(|lesson| lesson.kind)
                .unwrap_or(MediaKind::Audio)
        } else {
            MediaKind::Audio
        }
    }

    fn rebuild_dock_state(&mut self) {
        self.dock_state = dock_state_for_kind(
            self.show_media_library,
            self.show_segmentation,
            self.show_assistant,
            self.dock_media_kind(),
        );
    }

    fn render_tab(&mut self, ui: &mut egui::Ui, tab: DockTab) {
        match tab {
            DockTab::Player => self.ui_player(ui),
            DockTab::MediaLibrary => self.ui_media_library(ui),
            DockTab::Segmentation => self.ui_segmentation(ui),
            DockTab::Dictation => self.ui_dictation(ui),
            DockTab::Assistant => self.ui_assistant(ui),
        }
    }

    fn dock_actions(&mut self, ui: &mut egui::Ui, tab: DockTab, _caption: &str) {
        if self.maximized.is_some() {
            return;
        }
        let can_collapse = matches!(
            tab,
            DockTab::MediaLibrary | DockTab::Segmentation | DockTab::Assistant
        );
        let can_toggle_video = tab == DockTab::Player
            && self
                .project
                .active()
                .is_some_and(|lesson| lesson.kind == MediaKind::Video);
        let width = 27.0
            + if can_collapse { 24.0 } else { 0.0 }
            + if can_toggle_video { 24.0 } else { 0.0 };
        let position = egui::pos2(
            (ui.max_rect().right() - width - 3.0).max(ui.max_rect().left()),
            ui.max_rect().top() - 22.0,
        );
        egui::Area::new(egui::Id::new(("dock-top-actions", tab)))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .movable(false)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.horizontal(|ui| {
                    if can_toggle_video {
                        let tooltip = if self.show_video {
                            t!("actions.hide_video")
                        } else {
                            t!("actions.show_video")
                        };
                        if ui
                            .add(fa_small_button(FaIcon::Video))
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            self.show_video = !self.show_video;
                            if !self.show_video {
                                self.embedded_video.reset();
                            }
                            self.pending_dock_rebuild = true;
                        }
                    }
                    if can_collapse {
                        let (icon, tooltip) = if tab == DockTab::Assistant {
                            (FaIcon::Right, t!("actions.hide_right"))
                        } else {
                            (FaIcon::Left, t!("actions.hide_left"))
                        };
                        if ui
                            .add(fa_small_button(icon))
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            self.pending_collapse = Some(tab);
                        }
                    }
                    if ui
                        .add(fa_small_button(FaIcon::Maximize))
                        .on_hover_text(t!("icons.maximize"))
                        .clicked()
                    {
                        self.maximized = Some(tab);
                    }
                });
            });
    }

    fn sync_embedded_video(&mut self, ctx: &egui::Context) {
        let active_video = self.project.active().and_then(|lesson| {
            (self.show_video && lesson.kind == MediaKind::Video)
                .then(|| (PathBuf::from(&lesson.source_path), lesson.duration_ms))
        });
        let ffmpeg = self.config.ffmpeg();
        let playing = self.player.is_playing();
        let end_ms = if playing {
            self.player.range().1
        } else {
            active_video
                .as_ref()
                .map(|(_, duration_ms)| *duration_ms)
                .unwrap_or(1)
        };
        self.embedded_video.sync(
            ctx,
            ffmpeg.as_deref(),
            active_video.as_ref().map(|(path, _)| path.as_path()),
            playing,
            self.player.position_ms(),
            end_ms,
            self.project.playback_rate,
        );
    }

    fn ui_player(&mut self, ui: &mut egui::Ui) {
        self.dock_actions(ui, DockTab::Player, "VIDEO / AUDIO PLAYER");
        ui.spacing_mut().item_spacing.y = 2.0;
        let kind = self
            .project
            .active()
            .map(|lesson| lesson.kind)
            .unwrap_or(MediaKind::Audio);
        let duration = self
            .project
            .active()
            .map(|lesson| lesson.duration_ms)
            .unwrap_or(1);
        let (sentence_ranges, sentence_targets, word_ranges, word_targets) = self
            .project
            .active()
            .map(|lesson| {
                if !lesson.analyzed {
                    return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
                }
                let sentences: Vec<(u64, u64)> = lesson
                    .sentences
                    .iter()
                    .map(|sentence| (sentence.start_ms, sentence.end_ms))
                    .collect();
                let sentence_targets: Vec<LinkTarget> = lesson
                    .sentences
                    .iter()
                    .map(|sentence| LinkTarget::Sentence {
                        sentence_id: sentence.id.clone(),
                    })
                    .collect();
                let words: Vec<(u64, u64)> = lesson
                    .sentences
                    .iter()
                    .flat_map(|sentence| {
                        sentence
                            .words
                            .iter()
                            .map(|word| (word.start_ms, word.end_ms))
                    })
                    .collect();
                let word_targets: Vec<LinkTarget> = lesson
                    .sentences
                    .iter()
                    .flat_map(|sentence| {
                        sentence.words.iter().map(|word| LinkTarget::Word {
                            sentence_id: sentence.id.clone(),
                            word_id: word.id.clone(),
                        })
                    })
                    .collect();
                (sentences, sentence_targets, words, word_targets)
            })
            .unwrap_or_default();

        if kind == MediaKind::Video && self.show_video {
            let video_height = (ui.available_height() - 104.0)
                .max(150.0)
                .min(ui.available_width() * 9.0 / 16.0);
            self.embedded_video.show(ui, video_height);
            ui.add_space(3.0);
        }

        let position = self
            .media_scrub_position
            .unwrap_or_else(|| self.player.position_ms())
            .min(duration);
        let media_response = media_timeline(ui, duration, position, TIMELINE_HEIGHT)
            .on_hover_text(t!("player.seek"));
        if media_response.drag_started() {
            self.player.pause();
        }
        if media_response.dragged() {
            self.media_scrub_position = timeline_time_at(&media_response, duration);
        }
        if media_response.drag_stopped() {
            let start_ms = timeline_time_at(&media_response, duration)
                .or(self.media_scrub_position)
                .unwrap_or(position);
            self.media_scrub_position = None;
            self.play_media_from(start_ms);
        } else if media_response.clicked() {
            if let Some(start_ms) = timeline_time_at(&media_response, duration) {
                self.media_scrub_position = None;
                self.play_media_from(start_ms);
            }
        }
        timeline_time_labels(ui, duration, position);
        let sentence_response = timeline_strip(
            ui,
            duration,
            &sentence_ranges,
            Some(self.project.selected_sentence),
            TIMELINE_HEIGHT,
            TimelinePalette {
                background: Color32::from_rgb(229, 238, 251),
                fill: Color32::from_rgb(83, 138, 214),
                selected: Color32::from_rgb(35, 94, 178),
            },
        )
        .on_hover_text(t!("player.sentence_timeline"));
        if sentence_response.clicked() {
            if let Some(index) = timeline_segment_at(&sentence_response, duration, &sentence_ranges)
            {
                if let Some(target) = sentence_targets.get(index) {
                    self.play_target(target.clone());
                }
            }
        }
        let word_response = timeline_strip(
            ui,
            duration,
            &word_ranges,
            selected_flat_word_index(&self.project),
            TIMELINE_HEIGHT,
            TimelinePalette {
                background: Color32::from_rgb(252, 240, 222),
                fill: Color32::from_rgb(231, 157, 65),
                selected: Color32::from_rgb(193, 106, 24),
            },
        )
        .on_hover_text(t!("player.word_timeline"));
        if word_response.clicked() {
            if let Some(index) = timeline_segment_at(&word_response, duration, &word_ranges) {
                if let Some(target) = word_targets.get(index) {
                    self.play_target(target.clone());
                }
            }
        }
        bottom_aligned_control_bar(ui, |ui| {
            let mut volume = self.project.volume;
            let mut playback_rate = self.project.playback_rate;
            let mut loop_mode = self.project.loop_mode;
            let actions = player_control_strip(
                ui,
                self.player.is_playing(),
                &mut volume,
                &mut playback_rate,
                &mut loop_mode,
            );

            if actions.back {
                let _ = self
                    .player
                    .seek(self.player.position_ms().saturating_sub(3_000));
            }
            if actions.toggle_play {
                self.toggle_play();
            }
            if actions.forward {
                let _ = self
                    .player
                    .seek((self.player.position_ms() + 3_000).min(duration));
            }
            if actions.toggle_mute {
                self.toggle_audio_mute();
            } else if actions.volume_changed {
                self.set_audio_volume(volume);
            }
            self.project.playback_rate = playback_rate;
            if actions.rate_changed {
                if let Err(error) = self.player.set_rate(playback_rate) {
                    self.status = error;
                }
                self.dirty = true;
            }
            self.set_loop_mode(loop_mode);
        });
    }

    fn ui_media_library(&mut self, ui: &mut egui::Ui) {
        self.dock_actions(ui, DockTab::MediaLibrary, "MEDIA LIST");
        if ui
            .add(fa_button(FaIcon::Open))
            .on_hover_text(t!("actions.open_media"))
            .clicked()
        {
            self.open_media_dialog();
        }
        ui.add_space(6.0);
        let mut selected = None;
        let mut delete_requested = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, lesson) in self.project.lessons.iter().enumerate() {
                let active = index == self.project.active_lesson;
                let analyzing = self.analysis_running && self.analysis_lesson_index == Some(index);
                let row_width = ui.available_width();
                let response = egui::Frame::group(ui.style())
                    .fill(if active {
                        Color32::from_rgb(220, 244, 233)
                    } else {
                        Color32::WHITE
                    })
                    .stroke(Stroke::new(
                        1.0_f32,
                        if active {
                            Color32::from_rgb(29, 159, 94)
                        } else {
                            Color32::from_rgb(200, 214, 209)
                        },
                    ))
                    .inner_margin(8)
                    .show(ui, |ui| {
                        ui.set_min_width((row_width - 18.0).max(1.0));
                        ui.label(RichText::new(&lesson.title).strong().size(10.0));
                        ui.label(
                            RichText::new(format!(
                                "{} · {}",
                                lesson.kind.label(),
                                format_time(lesson.duration_ms)
                            ))
                            .monospace()
                            .size(9.0)
                            .color(Color32::from_rgb(84, 108, 100)),
                        );
                        ui.label(
                            RichText::new(if analyzing {
                                t!("media.whisper_analyzing")
                            } else if lesson.analyzed {
                                t!("media.whisper_analyzed")
                            } else {
                                t!("media.not_analyzed")
                            })
                            .size(9.0)
                            .color(if analyzing {
                                Color32::from_rgb(24, 99, 160)
                            } else if lesson.analyzed {
                                Color32::from_rgb(31, 154, 91)
                            } else {
                                Color32::from_rgb(170, 112, 29)
                            }),
                        );
                    })
                    .response
                    .interact(egui::Sense::click());
                if response.clicked() {
                    selected = Some(index);
                }
                response.context_menu(|ui| {
                    ui.label(RichText::new(&lesson.title).strong());
                    ui.separator();
                    let delete = ui.add_enabled(
                        !self.analysis_running,
                        egui::Button::image_and_text(
                            fa_image(FaIcon::Delete, 12.0),
                            RichText::new(t!("media.delete")).color(Color32::from_rgb(184, 48, 48)),
                        )
                        .image_tint_follows_text_color(true),
                    );
                    if delete.clicked() {
                        delete_requested = Some(index);
                        ui.close();
                    }
                    if self.analysis_running {
                        ui.label(
                            RichText::new(t!("media.analysis_required"))
                                .size(9.0)
                                .color(Color32::from_rgb(126, 126, 126)),
                        );
                    } else {
                        ui.label(
                            RichText::new(t!("media.files_kept"))
                                .size(9.0)
                                .color(Color32::from_rgb(84, 108, 100)),
                        );
                    }
                });
                ui.add_space(5.0);
            }
        });
        if let Some(index) = delete_requested {
            self.remove_media_from_library(index);
            return;
        }
        if let Some(index) = selected {
            self.project.select_lesson(index);
            self.current_play_target = None;
            self.show_video = self
                .project
                .active()
                .is_some_and(|lesson| lesson.kind == MediaKind::Video);
            self.media_scrub_position = None;
            self.player.stop();
            self.embedded_video.reset();
            self.pending_dock_rebuild = true;
            if self.project.active().is_some_and(|lesson| !lesson.analyzed) {
                self.start_analysis();
            }
        }
    }

    fn ui_segmentation(&mut self, ui: &mut egui::Ui) {
        self.dock_actions(
            ui,
            DockTab::Segmentation,
            "MEDIA → SENTENCES → PHRASES / WORDS",
        );
        let active_index = self.project.active_lesson;
        let analyzed = self.project.active().is_some_and(|lesson| lesson.analyzed);
        let analyzing_active =
            self.analysis_running && self.analysis_lesson_index == Some(active_index);
        let button_text = if analyzing_active {
            t!("segmentation.analyzing")
        } else if self.analysis_running {
            t!("segmentation.another_analyzing")
        } else if analyzed {
            t!("actions.rerun_segmentation")
        } else {
            t!("actions.intelligent_segmentation")
        };
        let (analyze_clicked, tips_clicked) = ui
            .horizontal(|ui| {
                let analyze_clicked = ui
                    .add_enabled(!self.analysis_running, fa_button(FaIcon::Analyze))
                    .on_hover_text(button_text)
                    .clicked();
                let tips_clicked = ui
                    .add(fa_button(FaIcon::Info).selected(self.show_segmentation_tips))
                    .on_hover_text(t!("segmentation.show_tips"))
                    .clicked();
                (analyze_clicked, tips_clicked)
            })
            .inner;
        if tips_clicked {
            self.show_segmentation_tips = !self.show_segmentation_tips;
        }
        if analyze_clicked {
            self.start_analysis();
        }
        if self.analysis_running {
            let elapsed = self
                .analysis_started_at
                .map_or(Duration::ZERO, |started| started.elapsed());
            let activity = analysis_activity(elapsed);
            let analysis_title = self
                .analysis_lesson_index
                .and_then(|index| self.project.lessons.get(index))
                .map(|lesson| lesson.title.as_str())
                .unwrap_or("media");
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(format!(
                        "{} Alive · elapsed {}",
                        activity.frame, activity.elapsed
                    ))
                    .size(10.0)
                    .color(Color32::from_rgb(24, 99, 160)),
                );
            });
            ui.label(
                RichText::new(t!("segmentation.analyzing_media", media = analysis_title))
                    .size(9.0)
                    .color(Color32::from_rgb(45, 84, 125)),
            );
            ui.label(
                RichText::new(t!("segmentation.tip", tip = activity.tip))
                    .size(9.0)
                    .color(Color32::from_rgb(74, 105, 142)),
            );
        } else if !analyzed {
            let error = self
                .analysis_error
                .as_ref()
                .and_then(|(index, message)| (*index == active_index).then_some(message.as_str()));
            if let Some(error) = error {
                ui.label(
                    RichText::new(t!("segmentation.failed", error = error))
                        .size(10.0)
                        .color(Color32::from_rgb(185, 45, 45)),
                );
            } else {
                ui.label(
                    RichText::new(t!("segmentation.not_analyzed"))
                        .size(10.0)
                        .color(Color32::from_rgb(168, 104, 25)),
                );
            }
        }
        ui.add_space(5.0);
        let mut select_sentence = None;
        let mut select_word = None;
        let mut play_target = None;
        let selected_sentence = self.project.selected_sentence;
        let selected_word = self.project.selected_word;
        let title = self
            .project
            .active()
            .map(|lesson| lesson.title.clone())
            .unwrap_or_else(|| t!("segmentation.no_media").into_owned());
        if analyzed || (analyzing_active && self.analysis_sentences_ready) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::CollapsingHeader::new(RichText::new(title).strong())
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(lesson) = self.project.active() {
                            for (sentence_index, sentence) in lesson.sentences.iter().enumerate() {
                                let sentence_label = segment_tree_label(
                                    'S',
                                    sentence_index + 1,
                                    sentence.start_ms,
                                    sentence.end_ms,
                                );
                                let header = egui::CollapsingHeader::new(
                                    RichText::new(sentence_label).color(segment_text_color(
                                        'S',
                                        sentence_index == selected_sentence,
                                    )),
                                )
                                .id_salt(("sentence-tree", sentence_index))
                                .default_open(sentence_index == selected_sentence)
                                .show(ui, |ui| {
                                    for (word_index, word) in sentence.words.iter().enumerate() {
                                        let selected = sentence_index == selected_sentence
                                            && selected_word == Some(word_index);
                                        let response = segment_response(
                                            ui.selectable_label(
                                                selected,
                                                RichText::new(segment_tree_label(
                                                    'W',
                                                    word_index + 1,
                                                    word.start_ms,
                                                    word.end_ms,
                                                ))
                                                .color(segment_text_color('W', selected)),
                                            ),
                                            self.show_segmentation_tips,
                                            &word.label,
                                        );
                                        if response.clicked() {
                                            select_sentence = Some(sentence_index);
                                            select_word = Some(word_index);
                                        }
                                        if segment_play_requested(&response) {
                                            play_target = Some(LinkTarget::Word {
                                                sentence_id: sentence.id.clone(),
                                                word_id: word.id.clone(),
                                            });
                                        }
                                    }
                                });
                                let response = segment_response(
                                    header.header_response,
                                    self.show_segmentation_tips,
                                    &sentence.text,
                                );
                                if response.clicked() {
                                    select_sentence = Some(sentence_index);
                                    select_word = sentence.words.first().map(|_| 0);
                                }
                                if segment_play_requested(&response) {
                                    play_target = Some(LinkTarget::Sentence {
                                        sentence_id: sentence.id.clone(),
                                    });
                                }
                            }
                        }
                    });
            });
        }
        if let Some(sentence) = select_sentence {
            self.project.selected_sentence = sentence;
            self.project.selected_word = select_word;
        }
        if let Some(target) = play_target {
            self.play_target(target);
        }
    }

    fn ui_dictation(&mut self, ui: &mut egui::Ui) {
        #[derive(Clone, Copy)]
        enum ToolbarAction {
            Format(TextFormatCommand),
            Heading(usize),
            Link,
            Copy,
            Cut,
            Paste,
            FindNext,
            Replace,
            ReplaceAll,
        }

        self.dock_actions(ui, DockTab::Dictation, "WYSIWYG MARKDOWN DICTATION");
        let Some((lesson_id, markdown)) = self
            .project
            .active()
            .map(|lesson| (lesson.id.clone(), lesson.markdown.clone()))
        else {
            return;
        };
        self.dictation_editor.ensure_document(lesson_id, &markdown);
        let source_mode = self.dictation_editor.source_mode();
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(!source_mode, |ui| {
                if format_w_button(
                    ui,
                    t!("dictation.underline"),
                    RichText::new("W").underline(),
                )
                .clicked()
                {
                    action = Some(ToolbarAction::Format(TextFormatCommand::Underline));
                }
                if format_w_button(
                    ui,
                    t!("dictation.blue_text"),
                    RichText::new("W").color(Color32::BLUE),
                )
                .clicked()
                {
                    action = Some(ToolbarAction::Format(TextFormatCommand::Blue));
                }
                if format_w_button(
                    ui,
                    t!("dictation.red_text"),
                    RichText::new("W").color(Color32::RED),
                )
                .clicked()
                {
                    action = Some(ToolbarAction::Format(TextFormatCommand::Red));
                }
                if format_w_button(ui, t!("dictation.bold"), RichText::new("W").strong()).clicked()
                {
                    action = Some(ToolbarAction::Format(TextFormatCommand::Bold));
                }
                if format_w_button(
                    ui,
                    t!("dictation.orange_crossline"),
                    RichText::new("W")
                        .color(Color32::from_rgb(230, 126, 34))
                        .strikethrough(),
                )
                .clicked()
                {
                    action = Some(ToolbarAction::Format(TextFormatCommand::OrangeCrossline));
                }

                if font_size_selector(ui, &mut self.dictation_font_size) {
                    action = Some(ToolbarAction::Format(TextFormatCommand::FontSize(
                        self.dictation_font_size,
                    )));
                }

                if paragraph_style_selector(ui, &mut self.dictation_heading_level) {
                    if self.dictation_heading_level > 0 {
                        self.dictation_font_size = DEFAULT_MARKDOWN_FONT_SIZE;
                    }
                    action = Some(ToolbarAction::Heading(self.dictation_heading_level));
                }

                if ui
                    .add(fa_button(FaIcon::Copy))
                    .on_hover_text(t!("dictation.copy"))
                    .clicked()
                {
                    action = Some(ToolbarAction::Copy);
                }
                if ui
                    .add(fa_button(FaIcon::Cut))
                    .on_hover_text(t!("dictation.cut"))
                    .clicked()
                {
                    action = Some(ToolbarAction::Cut);
                }
                if ui
                    .add(fa_button(FaIcon::Paste))
                    .on_hover_text(t!("dictation.paste"))
                    .clicked()
                {
                    action = Some(ToolbarAction::Paste);
                }
                if ui
                    .add(fa_button(FaIcon::Search).selected(self.show_dictation_find_replace))
                    .on_hover_text(t!("dictation.search_replace"))
                    .clicked()
                {
                    self.show_dictation_find_replace = !self.show_dictation_find_replace;
                }
                if ui
                    .add(fa_button(FaIcon::Link))
                    .on_hover_text(t!("actions.link_segment"))
                    .clicked()
                {
                    action = Some(ToolbarAction::Link);
                }
            });
            if ui
                .add(fa_button(FaIcon::Save))
                .on_hover_text(t!("actions.save_project"))
                .clicked()
            {
                self.save_project();
            }
            if source_mode_button(ui, source_mode).clicked() {
                self.dictation_editor.set_source_mode(!source_mode);
            }
        });
        if self.show_dictation_find_replace && !source_mode {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.dictation_find)
                        .desired_width(130.0)
                        .hint_text(t!("dictation.find_hint")),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.dictation_replace)
                        .desired_width(130.0)
                        .hint_text(t!("dictation.replace_hint")),
                );
                if ui.button(t!("dictation.find_next")).clicked() {
                    action = Some(ToolbarAction::FindNext);
                }
                if ui.button(t!("dictation.replace")).clicked() {
                    action = Some(ToolbarAction::Replace);
                }
                if ui.button(t!("dictation.replace_all")).clicked() {
                    action = Some(ToolbarAction::ReplaceAll);
                }
            });
        }
        ui.label(
            RichText::new(t!("dictation.help"))
                .size(9.0)
                .color(Color32::from_rgb(24, 99, 210)),
        );

        let mut document_changed = false;
        if let Some(action) = action {
            let result = match action {
                ToolbarAction::Format(TextFormatCommand::FontSize(pixels)) => {
                    self.dictation_editor.set_font_size(pixels).map(|changed| {
                        document_changed |= changed;
                        if changed {
                            "Text formatting updated"
                        } else {
                            "Font size set for new text"
                        }
                        .to_owned()
                    })
                }
                ToolbarAction::Format(command) => self
                    .dictation_editor
                    .toggle_selection_format(command)
                    .map(|changed| {
                        document_changed |= changed;
                        "Text formatting updated".to_owned()
                    }),
                ToolbarAction::Heading(level) => self
                    .dictation_editor
                    .set_selection_heading(level)
                    .map(|changed| {
                        document_changed |= changed;
                        "Paragraph style updated".to_owned()
                    }),
                ToolbarAction::Link => {
                    self.link_selection();
                    Ok(self.status.clone())
                }
                ToolbarAction::Copy => self.dictation_editor.selected_text().map(|text| {
                    ui.ctx().copy_text(text);
                    "Selection copied".to_owned()
                }),
                ToolbarAction::Cut => self.dictation_editor.cut_selection().map(|text| {
                    ui.ctx().copy_text(text);
                    document_changed = true;
                    "Selection cut".to_owned()
                }),
                ToolbarAction::Paste => self
                    .dictation_editor
                    .request_paste(ui.ctx())
                    .map(|()| "Paste requested".to_owned()),
                ToolbarAction::FindNext => self
                    .dictation_editor
                    .find_next(&self.dictation_find)
                    .map(|found| {
                        if found {
                            "Next match selected"
                        } else {
                            "No match found"
                        }
                        .to_owned()
                    }),
                ToolbarAction::Replace => self
                    .dictation_editor
                    .replace_current(&self.dictation_find, &self.dictation_replace)
                    .map(|changed| {
                        document_changed |= changed;
                        "Selection replaced".to_owned()
                    }),
                ToolbarAction::ReplaceAll => self
                    .dictation_editor
                    .replace_all(&self.dictation_find, &self.dictation_replace)
                    .map(|count| {
                        document_changed |= count > 0;
                        format!("Replaced {count} matches")
                    }),
            };
            match result {
                Ok(status) => self.status = status,
                Err(message) => self.status = message.into(),
            }
        }
        if document_changed {
            if let Some(lesson) = self.project.active_mut() {
                lesson.markdown = self.dictation_editor.markdown().to_owned();
            }
            self.dirty = true;
        }
        ui.add_space(5.0);
        let output = self.dictation_editor.show(ui);
        if output.changed {
            if let Some(lesson) = self.project.active_mut() {
                lesson.markdown = self.dictation_editor.markdown().to_owned();
            }
            self.dirty = true;
        }
        if let Some(target) = output.activated_link {
            self.play_target(target);
        }
    }

    fn ui_assistant(&mut self, ui: &mut egui::Ui) {
        self.dock_actions(ui, DockTab::Assistant, "ASR · CONVERSATION · TTS");
        ui.horizontal(|ui| {
            ui.add(egui::Image::new((self.bibi.id(), Vec2::new(52.0, 52.0))).corner_radius(26));
            ui.vertical(|ui| {
                ui.label(RichText::new(t!("assistant.companion")).strong());
                ui.label(
                    RichText::new(if self.analysis_running {
                        t!("assistant.listening")
                    } else {
                        t!("assistant.ready")
                    })
                    .size(9.0)
                    .color(Color32::from_rgb(28, 150, 86)),
                );
            });
        });
        ui.horizontal(|ui| {
            if ui
                .add(fa_button(FaIcon::Microphone))
                .on_hover_text(t!("actions.asr_audio"))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Audio", &["mp3", "wav", "flac", "ogg", "m4a"])
                    .pick_file()
                {
                    self.add_files(vec![path]);
                    self.start_analysis();
                }
            }
            if ui
                .add(fa_button(FaIcon::Volume))
                .on_hover_text(t!("actions.speak_response"))
                .clicked()
            {
                let text = self
                    .chat
                    .iter()
                    .rev()
                    .find(|message| message.assistant)
                    .map(|message| message.text.clone())
                    .unwrap_or_default();
                match speak_text(&text) {
                    Ok(()) => self.status = "Speaking assistant response".into(),
                    Err(error) => self.status = error,
                }
            }
        });
        ui.separator();
        let available = (ui.available_height() - 90.0).max(120.0);
        egui::ScrollArea::vertical()
            .max_height(available)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.chat {
                    let fill = if message.assistant {
                        Color32::from_rgb(226, 245, 236)
                    } else {
                        Color32::from_rgb(235, 239, 247)
                    };
                    egui::Frame::new()
                        .fill(fill)
                        .inner_margin(8)
                        .corner_radius(5)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(if message.assistant {
                                    std::borrow::Cow::Borrowed("Bibi")
                                } else {
                                    t!("assistant.you")
                                })
                                .strong()
                                .size(9.0)
                                .color(Color32::from_rgb(44, 113, 87)),
                            );
                            ui.label(&message.text);
                        });
                    ui.add_space(5.0);
                }
            });
        ui.separator();
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.assistant_input)
                .hint_text(t!("assistant.input_hint"))
                .desired_rows(2),
        );
        let send = ui
            .add(fa_button(FaIcon::Send))
            .on_hover_text(t!("actions.send_message"))
            .clicked()
            || (response.has_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl));
        if send && !self.assistant_input.trim().is_empty() {
            let prompt = std::mem::take(&mut self.assistant_input);
            let segment = self
                .project
                .selected_sentence()
                .map(|sentence| sentence.text.clone())
                .unwrap_or_else(|| t!("assistant.no_sentence").into_owned());
            self.chat.push(ChatMessage {
                assistant: false,
                text: prompt,
            });
            self.chat.push(ChatMessage { assistant: true, text: format!("Current practice segment: “{segment}”\n\nListen once at normal speed, repeat at 0.75×, then type it from memory. I can replay the selected sentence or any word from the segmentation tree.") });
        }
    }

    fn open_current_workspace(&mut self) {
        if let Some(lesson) = self.project.active() {
            open_in_file_manager(Path::new(&lesson.workspace_path));
        }
    }

    fn ui_about(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        egui::Window::new(t!("about.title"))
            .open(&mut open)
            .collapsible(false)
            .default_width(680.0)
            .show(ctx, |ui| {
                ui.add(egui::Image::new((self.logo.id(), Vec2::new(340.0, 57.0))));
                ui.heading(t!("about.heading"));
                ui.label(t!("about.stack"));
                ui.label(t!(
                    "about.config",
                    path = self.config.path.display().to_string()
                ));
                ui.separator();
                ui.label(RichText::new(t!("about.support")).strong());
                let width = ui.available_width();
                ui.add(
                    egui::Image::new((self.sponsor.id(), Vec2::new(width, width * 0.472)))
                        .corner_radius(7),
                );
            });
        self.show_about = open;
    }

    fn drive_qa_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.qa_screenshot_path.clone() else {
            return;
        };
        if self.analysis_running {
            return;
        }
        if self.qa_autoplay && !self.qa_autoplay_started {
            let target = std::env::var("BIBIPARROT_QA_WORD").ok().and_then(|value| {
                let (sentence, word) = value.split_once(':')?;
                let sentence = self
                    .project
                    .active()?
                    .sentences
                    .get(sentence.parse::<usize>().ok()?)?;
                let word = sentence.words.get(word.parse::<usize>().ok()?)?;
                Some(LinkTarget::Word {
                    sentence_id: sentence.id.clone(),
                    word_id: word.id.clone(),
                })
            });
            let target = target.or_else(|| {
                let index = std::env::var("BIBIPARROT_QA_SENTENCE")
                    .ok()?
                    .parse::<usize>()
                    .ok()?;
                Some(LinkTarget::Sentence {
                    sentence_id: self.project.active()?.sentences.get(index)?.id.clone(),
                })
            });
            if let Some(target) = target {
                self.play_target(target);
            } else {
                self.toggle_play();
            }
            self.qa_autoplay_started = true;
        }
        if std::env::var_os("BIBIPARROT_QA_WAIT_PLAYBACK").is_some() && self.player.is_playing() {
            return;
        }
        let screenshot = ctx.input(|input| {
            input.raw.events.iter().find_map(|event| {
                if let egui::Event::Screenshot { image, .. } = event {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(image) = screenshot {
            let rgba: Vec<u8> = image
                .pixels
                .iter()
                .flat_map(|pixel| pixel.to_array())
                .collect();
            if image::save_buffer(
                &path,
                &rgba,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ColorType::Rgba8,
            )
            .is_ok()
            {
                let report = serde_json::json!({
                    "range": self.player.range(),
                    "position": self.player.position_ms(),
                    "playing": self.player.is_playing(),
                    "status": self.status,
                });
                let _ = fs::write(path.with_extension("playback.json"), report.to_string());
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            self.qa_screenshot_path = None;
            return;
        }
        self.qa_frame_count = self.qa_frame_count.saturating_add(1);
        let capture_frame = if self.qa_autoplay { 40 } else { 8 };
        if self.qa_frame_count >= capture_frame && !self.qa_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                "design-qa",
            )));
            self.qa_requested = true;
        }
    }
}

impl TabViewer for BibiParrotApp {
    type Tab = DockTab;
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        self.render_tab(ui, *tab);
    }
}

impl eframe::App for BibiParrotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Err(error) = self.player.tick() {
            self.status = error;
        }
        self.sync_embedded_video(ctx);
        self.poll_analysis();
        ctx.request_repaint_after(Duration::from_millis(
            if self.analysis_running || self.player.is_playing() {
                33
            } else {
                250
            },
        ));
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        if !dropped.is_empty() {
            self.add_files(dropped);
        }
        let keyboard_free = !ctx.wants_keyboard_input();
        if keyboard_free && ctx.input(|input| input.key_pressed(egui::Key::Space)) {
            self.toggle_play();
        }
        if keyboard_free && ctx.input(|input| input.key_pressed(egui::Key::ArrowLeft)) {
            let _ = self
                .player
                .seek(self.player.position_ms().saturating_sub(3_000));
        }
        if keyboard_free && ctx.input(|input| input.key_pressed(egui::Key::ArrowRight)) {
            let _ = self.player.seek(self.player.position_ms() + 3_000);
        }
        if ctx.input_mut(|input| {
            input.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::S,
            ))
        }) {
            self.save_project();
        }
        self.menu_bar(ctx);
        self.toolbar(ctx);
        self.status_bar(ctx);
        self.show_docks(ctx);
        if self.show_about {
            self.ui_about(ctx);
        }
        self.drive_qa_screenshot(ctx);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_project();
    }
}

#[cfg(test)]
fn default_dock_state() -> DockState<DockTab> {
    dock_state_with_visibility(true, true, true)
}

#[cfg(test)]
fn dock_state_with_visibility(
    show_media_library: bool,
    show_segmentation: bool,
    show_assistant: bool,
) -> DockState<DockTab> {
    dock_state_for_kind(
        show_media_library,
        show_segmentation,
        show_assistant,
        MediaKind::Audio,
    )
}

fn dock_state_for_kind(
    show_media_library: bool,
    show_segmentation: bool,
    show_assistant: bool,
    kind: MediaKind,
) -> DockState<DockTab> {
    let mut state = DockState::new(vec![DockTab::Player]);
    let surface = state.main_surface_mut();
    let player_fraction = player_dock_fraction(kind);
    let [_player, mut editor] =
        surface.split_below(NodeIndex::root(), player_fraction, vec![DockTab::Dictation]);
    if show_assistant {
        [editor, _] = surface.split_right(editor, 0.78, vec![DockTab::Assistant]);
    }
    if show_media_library {
        [editor, _] = surface.split_left(editor, 0.20, vec![DockTab::MediaLibrary]);
    }
    if show_segmentation {
        let _ = surface.split_left(editor, 0.30, vec![DockTab::Segmentation]);
    }
    state
}

fn collapsed_dock_button(ui: &mut egui::Ui, tab: DockTab) -> egui::Response {
    let (triangle, action_label) = match tab {
        DockTab::MediaLibrary => ("▶", t!("actions.show_media_library")),
        DockTab::Segmentation => ("▶", t!("actions.show_segmentation")),
        DockTab::Assistant => ("◀", t!("actions.show_assistant")),
        DockTab::Player | DockTab::Dictation => unreachable!("core docks cannot be collapsed"),
    };
    let action_label = action_label.into_owned();
    let response = ui
        .add_sized(
            [22.0, 26.0],
            egui::Button::new(
                RichText::new(triangle)
                    .size(12.0)
                    .color(collapsed_dock_triangle_color(tab)),
            )
            .frame(false),
        )
        .on_hover_text(&action_label);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            action_label.clone(),
        )
    });
    response
}

fn collapsed_left_dock_buttons(
    ui: &mut egui::Ui,
    show_media_library: &mut bool,
    show_segmentation: &mut bool,
) -> bool {
    let mut restored = false;
    ui.vertical_centered(|ui| {
        if !*show_media_library && collapsed_dock_button(ui, DockTab::MediaLibrary).clicked() {
            *show_media_library = true;
            restored = true;
        }
        if !*show_segmentation && collapsed_dock_button(ui, DockTab::Segmentation).clicked() {
            *show_segmentation = true;
            restored = true;
        }
    });
    restored
}

fn collapsed_right_dock_buttons(ui: &mut egui::Ui, show_assistant: &mut bool) -> bool {
    if !*show_assistant && collapsed_dock_button(ui, DockTab::Assistant).clicked() {
        *show_assistant = true;
        return true;
    }
    false
}

fn player_dock_fraction(kind: MediaKind) -> f32 {
    match kind {
        MediaKind::Audio => 0.14,
        MediaKind::Video => 0.58,
    }
}

fn collapsed_dock_control_top(available: egui::Rect, kind: MediaKind) -> f32 {
    available.top() + available.height() * player_dock_fraction(kind) + 30.0
}

fn collapsed_dock_triangle_color(tab: DockTab) -> Color32 {
    match tab {
        DockTab::MediaLibrary => Color32::from_rgba_unmultiplied(22, 142, 86, 176),
        DockTab::Segmentation => Color32::from_rgba_unmultiplied(37, 99, 184, 176),
        DockTab::Assistant => Color32::from_rgba_unmultiplied(139, 92, 246, 176),
        DockTab::Player | DockTab::Dictation => unreachable!("core docks cannot be collapsed"),
    }
}

fn segment_tree_label(level: char, index: usize, start_ms: u64, end_ms: u64) -> String {
    format!(
        "{}–{}  {level}{index}",
        format_time(start_ms),
        format_time(end_ms)
    )
}

fn segment_text_color(level: char, selected: bool) -> Color32 {
    match (level, selected) {
        ('S', true) => Color32::from_rgb(20, 135, 77),
        ('S', false) => Color32::from_rgb(35, 91, 145),
        ('W', true) => Color32::from_rgb(186, 92, 18),
        ('W', false) => Color32::from_rgb(151, 83, 24),
        _ => Color32::DARK_GRAY,
    }
}

fn segment_response(response: egui::Response, show_tips: bool, text: &str) -> egui::Response {
    if show_tips {
        response.on_hover_text(text)
    } else {
        response
    }
}

fn segment_play_requested(response: &egui::Response) -> bool {
    response.double_clicked() || response.secondary_clicked()
}

fn selected_flat_word_index(project: &Project) -> Option<usize> {
    let selected_word = project.selected_word?;
    let lesson = project.active()?;
    let preceding = lesson
        .sentences
        .iter()
        .take(project.selected_sentence)
        .map(|sentence| sentence.words.len())
        .sum::<usize>();
    Some(preceding + selected_word)
}

fn timeline_strip(
    ui: &mut egui::Ui,
    duration_ms: u64,
    ranges: &[(u64, u64)],
    selected: Option<usize>,
    height: f32,
    palette: TimelinePalette,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::click(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2, palette.background);
    let duration = duration_ms.max(1) as f32;
    for (index, &(start_ms, end_ms)) in ranges.iter().enumerate() {
        let left = rect.left() + rect.width() * start_ms.min(duration_ms) as f32 / duration;
        let right = rect.left() + rect.width() * end_ms.min(duration_ms) as f32 / duration;
        let segment = egui::Rect::from_min_max(
            egui::pos2(left + 1.0, rect.top() + 1.0),
            egui::pos2((right - 1.0).max(left + 2.0), rect.bottom() - 1.0),
        );
        painter.rect_filled(
            segment,
            1,
            if selected == Some(index) {
                palette.selected
            } else {
                palette.fill
            },
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn media_timeline(
    ui: &mut egui::Ui,
    duration_ms: u64,
    position_ms: u64,
    height: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2, Color32::from_rgb(223, 241, 232));
    let fraction = position_ms.min(duration_ms) as f32 / duration_ms.max(1) as f32;
    let played = egui::Rect::from_min_max(
        rect.left_top(),
        egui::pos2(rect.left() + rect.width() * fraction, rect.bottom()),
    );
    painter.rect_filled(played, 2, Color32::from_rgb(28, 170, 94));
    let marker_radius = (rect.height() * 0.36).max(2.8);
    let cursor_x = (rect.left() + rect.width() * fraction)
        .clamp(rect.left() + marker_radius, rect.right() - marker_radius);
    painter.vline(
        cursor_x,
        rect.y_range(),
        Stroke::new(1.0_f32, Color32::from_rgb(7, 87, 48)),
    );
    painter.circle_filled(
        egui::pos2(cursor_x, rect.center().y),
        marker_radius,
        Color32::WHITE,
    );
    painter.circle_stroke(
        egui::pos2(cursor_x, rect.center().y),
        marker_radius,
        Stroke::new(1.0_f32, Color32::from_rgb(7, 87, 48)),
    );
    let time_text = format_time(position_ms);
    let (text_position, alignment) = if fraction > 0.82 {
        (
            egui::pos2(cursor_x - marker_radius - 3.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
        )
    } else {
        (
            egui::pos2(cursor_x + marker_radius + 3.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
        )
    };
    painter.text(
        text_position,
        alignment,
        time_text,
        egui::FontId::monospace(7.0),
        Color32::from_rgb(7, 87, 48),
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn timeline_time_labels(ui: &mut egui::Ui, duration_ms: u64, position_ms: u64) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 10.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let font = egui::FontId::monospace(7.5);
    let color = ui.visuals().weak_text_color();
    painter.text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        "00:00.0",
        font.clone(),
        color,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("NOW: {}", format_time(position_ms)),
        font.clone(),
        Color32::from_rgb(7, 87, 48),
    );
    painter.text(
        rect.right_center(),
        egui::Align2::RIGHT_CENTER,
        format_time(duration_ms),
        font,
        color,
    );
}

fn timeline_time_at(response: &egui::Response, duration_ms: u64) -> Option<u64> {
    let pointer = response.interact_pointer_pos()?;
    Some(timeline_time_from_x(
        pointer.x,
        response.rect.left(),
        response.rect.width(),
        duration_ms,
    ))
}

fn timeline_time_from_x(x: f32, left: f32, width: f32, duration_ms: u64) -> u64 {
    let fraction = ((x - left) / width.max(1.0)).clamp(0.0, 1.0);
    (fraction * duration_ms as f32).round() as u64
}

fn timeline_segment_at(
    response: &egui::Response,
    duration_ms: u64,
    ranges: &[(u64, u64)],
) -> Option<usize> {
    let time = timeline_time_at(response, duration_ms)?;
    ranges
        .iter()
        .position(|&(start_ms, end_ms)| start_ms <= time && time <= end_ms)
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = Color32::from_rgb(235, 243, 240);
    visuals.window_fill = Color32::from_rgb(246, 250, 248);
    visuals.selection.bg_fill = Color32::from_rgb(25, 154, 88);
    visuals.widgets.active.bg_fill = Color32::from_rgb(27, 140, 83);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(218, 239, 230);
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(7.0, 6.0);
    style.spacing.button_padding = Vec2::new(9.0, 5.0);
    ctx.set_style(style);
}

fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let rgba = image::load_from_memory(bytes)
        .expect("valid embedded image")
        .to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    ctx.load_texture(name, color, egui::TextureOptions::LINEAR)
}

fn open_in_file_manager(path: &Path) {
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("explorer.exe");
        command.arg(path);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let _ = command.spawn();
}

fn speak_text(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("There is no assistant response to speak".into());
    }
    #[cfg(windows)]
    {
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command",
            "Add-Type -AssemblyName System.Speech; $voice = New-Object System.Speech.Synthesis.SpeechSynthesizer; $voice.Speak($env:BIBIPARROT_TTS_TEXT)"])
            .env("BIBIPARROT_TTS_TEXT", text).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Could not start Windows TTS: {error}"))
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("say")
            .arg(text)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Could not start macOS TTS: {error}"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("spd-say")
            .arg(text)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Could not start Linux TTS: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        kittest::{NodeT as _, Queryable as _},
        Harness,
    };
    use std::collections::HashSet;

    #[test]
    fn font_awesome_icons_are_embedded_as_unique_svg_resources() {
        let mut uris = HashSet::new();
        for icon in FaIcon::ALL {
            let svg = std::str::from_utf8(icon.bytes()).unwrap();
            assert!(svg.starts_with("<svg"), "invalid SVG for {icon:?}");
            assert!(svg.contains("<path"), "missing path for {icon:?}");
            assert!(uris.insert(icon.uri()), "duplicate URI for {icon:?}");
        }
    }

    #[test]
    fn icon_buttons_are_accessible_and_clickable_through_kittest() {
        #[derive(Default)]
        struct State {
            play_clicked: bool,
            save_clicked: bool,
        }

        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                egui_extras::install_image_loaders(ui.ctx());
                if ui.add(fa_button(FaIcon::Play)).clicked() {
                    state.play_clicked = true;
                }
                if ui.add(fa_button(FaIcon::Save)).clicked() {
                    state.save_clicked = true;
                }
            },
            State::default(),
        );

        let play = harness.get_by_label("Play");
        assert_eq!(play.accesskit_node().label().as_deref(), Some("Play"));
        play.click();
        harness.run();
        assert!(harness.state().play_clicked);
        assert!(!harness.state().save_clicked);

        harness.get_by_label("Save").click();
        harness.run();
        assert!(harness.state().save_clicked);
    }

    #[test]
    fn volume_button_switches_icon_and_restores_the_previous_level() {
        struct State {
            volume: u8,
            volume_before_mute: u8,
        }

        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                egui_extras::install_image_loaders(ui.ctx());
                if volume_button(ui, state.volume).clicked() {
                    toggle_muted_volume(&mut state.volume, &mut state.volume_before_mute);
                }
            },
            State {
                volume: 64,
                volume_before_mute: 64,
            },
        );

        harness.get_by_label("Volume").click();
        harness.run();
        assert_eq!(harness.state().volume, 0);
        assert!(harness.query_by_label("Muted").is_some());

        harness.get_by_label("Muted").click();
        harness.run();
        assert_eq!(harness.state().volume, 64);
        assert!(harness.query_by_label("Volume").is_some());
    }

    #[test]
    fn source_mode_toolbar_button_toggles_the_markdown_editor() {
        let mut editor = MarkdownEditor::default();
        editor.ensure_document("lesson", "## Heading");
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                egui_extras::install_image_loaders(ui.ctx());
                let source_mode = editor.source_mode();
                if source_mode_button(ui, source_mode).clicked() {
                    editor.set_source_mode(!source_mode);
                }
                editor.show(ui);
            },
            editor,
        );

        harness.get_by_label("Markdown source").click();
        harness.run();
        assert!(harness.state().source_mode());
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("## Heading")
        );

        harness.get_by_label("Markdown source").click();
        harness.run();
        assert!(!harness.state().source_mode());
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("Heading")
        );
    }

    #[test]
    fn underline_w_toolbar_button_formats_the_visible_selection() {
        let mut editor = MarkdownEditor::default();
        editor.ensure_document("lesson", "snow falls");
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                if format_w_button(
                    ui,
                    "Underline selected text",
                    RichText::new("W").underline(),
                )
                .clicked()
                {
                    editor
                        .toggle_selection_format(TextFormatCommand::Underline)
                        .unwrap();
                }
                editor.show(ui);
            },
            editor,
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.run();
        harness.get_by_label("Underline selected text").click();
        harness.run();

        assert_eq!(harness.state().markdown(), "<u>snow falls</u>");
    }

    #[test]
    fn font_size_selector_formats_the_visible_selection() {
        struct State {
            editor: MarkdownEditor,
            font_size: u16,
        }
        let mut editor = MarkdownEditor::default();
        editor.ensure_document("lesson", "snow falls");
        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                if font_size_selector(ui, &mut state.font_size) {
                    state
                        .editor
                        .toggle_selection_format(TextFormatCommand::FontSize(state.font_size))
                        .unwrap();
                }
                state.editor.show(ui);
            },
            State {
                editor,
                font_size: 14,
            },
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.run();
        harness.get_by_role(egui::accesskit::Role::ComboBox).click();
        harness.run();
        harness.get_by_label("18 px").click();
        harness.run();

        assert_eq!(
            harness.state().editor.markdown(),
            "<span style=\"font-size:18px\">snow falls</span>"
        );
    }

    #[test]
    fn heading_selector_formats_the_selected_line() {
        struct State {
            editor: MarkdownEditor,
            heading_level: usize,
        }
        let mut editor = MarkdownEditor::default();
        editor.ensure_document("lesson", "snow falls");
        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                if paragraph_style_selector(ui, &mut state.heading_level) {
                    state
                        .editor
                        .set_selection_heading(state.heading_level)
                        .unwrap();
                }
                state.editor.show(ui);
            },
            State {
                editor,
                heading_level: 0,
            },
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.run();
        harness.get_by_role(egui::accesskit::Role::ComboBox).click();
        harness.run();
        harness.get_by_label("H2").click();
        harness.run();

        assert_eq!(harness.state().editor.markdown(), "## snow falls");
    }

    #[test]
    fn font_size_selector_sets_the_size_for_newly_typed_text() {
        struct State {
            editor: MarkdownEditor,
            font_size: u16,
        }
        let mut editor = MarkdownEditor::default();
        editor.ensure_document("lesson", "");
        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                if font_size_selector(ui, &mut state.font_size) {
                    state.editor.set_font_size(state.font_size).unwrap();
                }
                state.editor.show(ui);
            },
            State {
                editor,
                font_size: 14,
            },
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        harness.get_by_role(egui::accesskit::Role::ComboBox).click();
        harness.run();
        harness.get_by_label("24 px").click();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .type_text("snow");
        harness.run();

        assert_eq!(
            harness.state().editor.markdown(),
            "<span style=\"font-size:24px\">snow</span>"
        );
    }

    #[test]
    fn heading_selector_styles_text_typed_without_a_selection() {
        struct State {
            editor: MarkdownEditor,
            heading_level: usize,
        }
        let mut editor = MarkdownEditor::default();
        editor.ensure_document("lesson", "");
        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                if paragraph_style_selector(ui, &mut state.heading_level) {
                    state
                        .editor
                        .set_selection_heading(state.heading_level)
                        .unwrap();
                }
                state.editor.show(ui);
            },
            State {
                editor,
                heading_level: 0,
            },
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        harness.get_by_role(egui::accesskit::Role::ComboBox).click();
        harness.run();
        harness.get_by_label("H3").click();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .type_text("snow");
        harness.run();

        assert_eq!(harness.state().editor.markdown(), "### snow");
    }

    #[test]
    fn media_controls_are_anchored_to_the_bottom_of_the_player_dock() {
        let harness = Harness::builder()
            .with_size(Vec2::new(480.0, 180.0))
            .build_ui(|ui| {
                egui_extras::install_image_loaders(ui.ctx());
                ui.label("Media timelines");
                bottom_aligned_control_bar(ui, |ui| {
                    ui.add(fa_button(FaIcon::Play));
                });
            });

        let dock_bottom = 180.0;
        let controls_bottom = harness.get_by_label("Play").rect().bottom();
        assert!(
            dock_bottom - controls_bottom <= 10.0,
            "control bar left a {} px bottom gap",
            dock_bottom - controls_bottom
        );
    }

    #[test]
    fn player_controls_stay_ordered_inside_a_narrow_dock() {
        struct State {
            volume: u8,
            playback_rate: f32,
            loop_mode: LoopMode,
        }

        let harness = Harness::builder()
            .with_size(Vec2::new(420.0, 60.0))
            .build_ui_state(
                |ui, state: &mut State| {
                    egui_extras::install_image_loaders(ui.ctx());
                    player_control_strip(
                        ui,
                        false,
                        &mut state.volume,
                        &mut state.playback_rate,
                        &mut state.loop_mode,
                    );
                },
                State {
                    volume: 100,
                    playback_rate: 1.0,
                    loop_mode: LoopMode::Off,
                },
            );

        let controls = [
            harness.get_by_label("Previous").rect(),
            harness.get_by_label("Play").rect(),
            harness.get_by_label("Next").rect(),
            harness.get_by_label("Volume").rect(),
            harness.get_by_label("Volume level").rect(),
            harness.get_by_label("Playback speed").rect(),
            harness.get_by_label("Playback loop mode").rect(),
        ];
        for pair in controls.windows(2) {
            assert!(
                pair[0].right() <= pair[1].left(),
                "player controls overlap: {:?} and {:?}",
                pair[0],
                pair[1]
            );
        }
        for control in &controls[..4] {
            assert!(
                control.width() <= 26.0,
                "player button is wider than the compact target: {control:?}"
            );
        }
        assert!(
            controls.last().unwrap().right() <= 420.0,
            "player controls overflow the dock: {:?}",
            controls.last().unwrap()
        );
    }

    #[test]
    fn player_controls_collapse_secondary_inputs_before_overflowing() {
        struct State {
            volume: u8,
            playback_rate: f32,
            loop_mode: LoopMode,
        }

        let harness = Harness::builder()
            .with_size(Vec2::new(280.0, 60.0))
            .build_ui_state(
                |ui, state: &mut State| {
                    egui_extras::install_image_loaders(ui.ctx());
                    player_control_strip(
                        ui,
                        false,
                        &mut state.volume,
                        &mut state.playback_rate,
                        &mut state.loop_mode,
                    );
                },
                State {
                    volume: 100,
                    playback_rate: 1.0,
                    loop_mode: LoopMode::Off,
                },
            );

        assert!(harness.query_by_label("Volume level").is_none());
        assert!(harness.query_by_label("Playback speed").is_none());
        assert!(harness.get_by_label("Playback loop mode").rect().right() <= 280.0);
    }

    #[test]
    fn loop_selector_changes_visible_playback_mode_through_kittest() {
        let mut harness = Harness::new_ui_state(
            |ui, mode: &mut LoopMode| {
                egui_extras::install_image_loaders(ui.ctx());
                loop_mode_selector(ui, "kittest-loop", mode, 88.0);
            },
            LoopMode::Off,
        );

        harness.get_by_label("Playback loop mode").click();
        harness.run();
        harness.get_by_label("Loop").click();
        harness.run();

        assert_eq!(*harness.state(), LoopMode::Loop);
        assert!(harness.query_by_label("Playback loop mode").is_some());
    }

    #[test]
    fn default_layout_contains_every_requested_dock() {
        let state = default_dock_state();
        let tabs: Vec<DockTab> = state.iter_all_tabs().map(|(_, tab)| *tab).collect();
        for expected in DockTab::ALL {
            assert!(tabs.contains(&expected), "missing {}", expected.title());
        }
        assert_eq!(tabs.len(), DockTab::ALL.len());
    }

    #[test]
    fn collapsed_side_docks_are_omitted_from_rebuilt_layout() {
        let state = dock_state_with_visibility(false, false, false);
        let tabs: Vec<DockTab> = state.iter_all_tabs().map(|(_, tab)| *tab).collect();
        assert_eq!(tabs, vec![DockTab::Player, DockTab::Dictation]);
    }

    #[test]
    fn collapsed_left_docks_can_be_restored_individually_from_the_edge_rail() {
        #[derive(Default)]
        struct Visibility {
            media_library: bool,
            segmentation: bool,
        }
        let mut harness = Harness::new_ui_state(
            |ui, visibility: &mut Visibility| {
                collapsed_left_dock_buttons(
                    ui,
                    &mut visibility.media_library,
                    &mut visibility.segmentation,
                );
            },
            Visibility::default(),
        );

        assert!(harness.query_by_label("Show Media Library").is_some());
        assert!(harness.query_by_label("Show Segmentation Tree").is_some());
        harness.get_by_label("Show Media Library").click();
        harness.run();

        assert!(harness.state().media_library);
        assert!(!harness.state().segmentation);
        assert!(harness.query_by_label("Show Media Library").is_none());
        assert!(harness.query_by_label("Show Segmentation Tree").is_some());
    }

    #[test]
    fn collapsed_assistant_can_be_restored_from_the_right_edge_rail() {
        let mut harness = Harness::new_ui_state(
            |ui, show_assistant: &mut bool| {
                collapsed_right_dock_buttons(ui, show_assistant);
            },
            false,
        );

        harness.get_by_label("Show AI Assistant").click();
        harness.run();

        assert!(*harness.state());
        assert!(harness.query_by_label("Show AI Assistant").is_none());
    }

    #[test]
    fn segment_tree_labels_put_timeline_before_short_id() {
        assert_eq!(
            segment_tree_label('S', 1, 200, 2_200),
            "00:00.2–00:02.2  S1"
        );
        assert_eq!(
            segment_tree_label('W', 7, 600, 1_400),
            "00:00.6–00:01.4  W7"
        );
    }

    #[test]
    fn sentence_and_word_rows_have_distinct_colors() {
        assert_ne!(
            segment_text_color('S', false),
            segment_text_color('W', false)
        );
        assert_ne!(segment_text_color('S', true), segment_text_color('W', true));
    }

    #[test]
    fn segment_tips_are_optional_and_secondary_click_requests_playback() {
        #[derive(Default)]
        struct State {
            played: bool,
        }
        let mut harness = Harness::new_ui_state(
            |ui, state: &mut State| {
                let response = segment_response(ui.button("00:00.0–00:00.5  W1"), true, "Hello");
                state.played |= segment_play_requested(&response);
            },
            State::default(),
        );

        harness.get_by_label_contains("W1").hover();
        harness.run();
        assert!(harness.query_by_label("Hello").is_some());
        harness.get_by_label_contains("W1").click_secondary();
        harness.run();
        assert!(harness.state().played);

        let mut disabled = Harness::new_ui(|ui| {
            segment_response(ui.button("00:00.0–00:00.5  W1"), false, "Hello");
        });
        disabled.get_by_label_contains("W1").hover();
        disabled.run();
        assert!(disabled.query_by_label("Hello").is_none());
    }

    #[test]
    fn video_media_gets_a_taller_player_dock_than_audio() {
        assert!(player_dock_fraction(MediaKind::Video) > 0.5);
        assert!(player_dock_fraction(MediaKind::Audio) < 0.25);
    }

    #[test]
    fn collapsed_dock_controls_begin_below_the_player() {
        let available = egui::Rect::from_min_size(egui::pos2(0.0, 100.0), egui::vec2(900.0, 700.0));
        for kind in [MediaKind::Audio, MediaKind::Video] {
            let player_bottom = available.top() + available.height() * player_dock_fraction(kind);
            assert!(collapsed_dock_control_top(available, kind) >= player_bottom + 24.0);
        }
    }

    #[test]
    fn collapsed_dock_triangles_use_distinct_colors() {
        let media = collapsed_dock_triangle_color(DockTab::MediaLibrary);
        let segmentation = collapsed_dock_triangle_color(DockTab::Segmentation);
        let assistant = collapsed_dock_triangle_color(DockTab::Assistant);
        assert_ne!(media, segmentation);
        assert_ne!(media, assistant);
        assert_ne!(segmentation, assistant);
    }

    #[test]
    fn collapsed_dock_triangles_are_translucent() {
        for tab in [
            DockTab::MediaLibrary,
            DockTab::Segmentation,
            DockTab::Assistant,
        ] {
            assert!(collapsed_dock_triangle_color(tab).a() < 255);
        }
    }

    #[test]
    fn timeline_pointer_maps_to_media_timestamp_and_clamps() {
        assert_eq!(timeline_time_from_x(50.0, 0.0, 100.0, 10_000), 5_000);
        assert_eq!(timeline_time_from_x(-5.0, 0.0, 100.0, 10_000), 0);
        assert_eq!(timeline_time_from_x(150.0, 0.0, 100.0, 10_000), 10_000);
    }

    #[test]
    fn analysis_activity_animates_and_cycles_tips() {
        let initial = analysis_activity(Duration::ZERO);
        let animated = analysis_activity(Duration::from_millis(250));
        let next_tip = analysis_activity(Duration::from_secs(4));
        let wrapped_tip = analysis_activity(Duration::from_secs(20));

        assert_ne!(initial.frame, animated.frame);
        assert_ne!(initial.tip, next_tip.tip);
        assert_eq!(initial.tip, wrapped_tip.tip);
        assert_eq!(initial.elapsed, "00:00");
        assert_eq!(
            analysis_activity(Duration::from_secs(3_723)).elapsed,
            "01:02:03"
        );
    }
}
