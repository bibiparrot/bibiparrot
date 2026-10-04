use eframe::egui::{self, Color32, FontId, TextureHandle, Vec2};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError, TrySendError},
        Arc, Mutex,
    },
    thread,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const FRAME_WIDTH: usize = 960;
const FRAME_HEIGHT: usize = 540;
const FRAME_BYTES: usize = FRAME_WIDTH * FRAME_HEIGHT * 4;

struct VideoFrame {
    rgba: Vec<u8>,
}

struct DecoderSession {
    child: Arc<Mutex<Child>>,
    cancel: Arc<AtomicBool>,
    receiver: Receiver<VideoFrame>,
}

impl DecoderSession {
    fn stop(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

impl Drop for DecoderSession {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Clone, PartialEq, Eq)]
struct DecoderKey {
    source: PathBuf,
    end_ms: u64,
    rate_percent: u16,
}

pub struct EmbeddedVideo {
    session: Option<DecoderSession>,
    texture: Option<TextureHandle>,
    source: Option<PathBuf>,
    key: Option<DecoderKey>,
    last_position_ms: Option<u64>,
    error: Option<String>,
}

impl EmbeddedVideo {
    pub fn new() -> Self {
        Self {
            session: None,
            texture: None,
            source: None,
            key: None,
            last_position_ms: None,
            error: None,
        }
    }

    pub fn reset(&mut self) {
        self.stop_session();
        self.texture = None;
        self.source = None;
        self.key = None;
        self.last_position_ms = None;
        self.error = None;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sync(
        &mut self,
        ctx: &egui::Context,
        ffmpeg: Option<&Path>,
        source: Option<&Path>,
        playing: bool,
        position_ms: u64,
        end_ms: u64,
        rate: f32,
    ) {
        let source_changed = self.source.as_deref() != source;
        if source_changed {
            self.reset();
            self.source = source.map(Path::to_path_buf);
        }
        if source.is_none() {
            return;
        }

        self.poll_frame(ctx);
        let Some(ffmpeg) = ffmpeg else {
            self.error = Some("FFmpeg is required for embedded video".into());
            self.stop_session();
            return;
        };
        let source = source.expect("checked above");

        if !playing {
            if self.texture.is_some() {
                self.stop_session();
            } else if self.session.is_none() {
                let poster_end = (position_ms + 1_000).min(end_ms).max(position_ms + 1);
                self.start_session(ffmpeg, source, position_ms, poster_end, 10.0);
            }
            self.last_position_ms = Some(position_ms);
            return;
        }

        let rate = rate.clamp(0.5, 2.0);
        let key = DecoderKey {
            source: source.to_path_buf(),
            end_ms,
            rate_percent: (rate * 100.0).round() as u16,
        };
        let jumped = self
            .last_position_ms
            .is_some_and(|previous| previous.abs_diff(position_ms) > 750);
        if self.session.is_none() || self.key.as_ref() != Some(&key) || jumped {
            self.start_session(ffmpeg, source, position_ms, end_ms, rate);
            self.key = Some(key);
        }
        self.last_position_ms = Some(position_ms);
    }

    pub fn show(&self, ui: &mut egui::Ui, height: f32) {
        let size = Vec2::new(ui.available_width(), height.max(120.0));
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4, Color32::from_rgb(7, 15, 12));

        if let Some(texture) = &self.texture {
            let scale =
                (rect.width() / FRAME_WIDTH as f32).min(rect.height() / FRAME_HEIGHT as f32);
            let image_size = Vec2::new(FRAME_WIDTH as f32, FRAME_HEIGHT as f32) * scale;
            let image_rect = egui::Rect::from_center_size(rect.center(), image_size);
            ui.put(
                image_rect,
                egui::Image::new((texture.id(), image_rect.size())),
            );
        } else {
            let message = self
                .error
                .as_deref()
                .unwrap_or("Video preview is loading inside BibiParrot…");
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                message,
                FontId::proportional(13.0),
                Color32::from_rgb(170, 202, 190),
            );
        }
    }

    fn poll_frame(&mut self, ctx: &egui::Context) {
        let mut latest = None;
        let mut disconnected = false;
        if let Some(session) = &self.session {
            loop {
                match session.receiver.try_recv() {
                    Ok(frame) => latest = Some(frame),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        if let Some(frame) = latest {
            let image =
                egui::ColorImage::from_rgba_unmultiplied([FRAME_WIDTH, FRAME_HEIGHT], &frame.rgba);
            if let Some(texture) = &mut self.texture {
                texture.set(image, egui::TextureOptions::LINEAR);
            } else {
                self.texture = Some(ctx.load_texture(
                    "embedded-video-frame",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
            self.error = None;
        }
        if disconnected {
            self.session = None;
        }
    }

    fn start_session(
        &mut self,
        ffmpeg: &Path,
        source: &Path,
        start_ms: u64,
        end_ms: u64,
        rate: f32,
    ) {
        self.stop_session();
        match spawn_decoder(ffmpeg, source, start_ms, end_ms, rate) {
            Ok(session) => {
                self.session = Some(session);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn stop_session(&mut self) {
        if let Some(session) = self.session.take() {
            session.stop();
        }
    }
}

impl Drop for EmbeddedVideo {
    fn drop(&mut self) {
        self.stop_session();
    }
}

fn spawn_decoder(
    ffmpeg: &Path,
    source: &Path,
    start_ms: u64,
    end_ms: u64,
    rate: f32,
) -> Result<DecoderSession, String> {
    let mut command = Command::new(ffmpeg);
    configure_decoder_command(&mut command, source, start_ms, end_ms, rate);
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start embedded video decoder: {error}"))?;
    let Some(mut stdout) = child.stdout.take() else {
        let _ = child.kill();
        return Err("FFmpeg did not provide embedded video frames".into());
    };
    let child = Arc::new(Mutex::new(child));
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_child = Arc::clone(&child);
    let worker_cancel = Arc::clone(&cancel);
    let (sender, receiver) = mpsc::sync_channel(2);
    thread::spawn(move || {
        let mut rgba = vec![0_u8; FRAME_BYTES];
        while !worker_cancel.load(Ordering::Acquire) {
            if stdout.read_exact(&mut rgba).is_err() {
                break;
            }
            match sender.try_send(VideoFrame { rgba }) {
                Ok(()) => rgba = vec![0_u8; FRAME_BYTES],
                Err(TrySendError::Full(frame)) => rgba = frame.rgba,
                Err(TrySendError::Disconnected(_)) => break,
            }
        }
        if let Ok(mut child) = worker_child.lock() {
            if worker_cancel.load(Ordering::Acquire) {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    });
    Ok(DecoderSession {
        child,
        cancel,
        receiver,
    })
}

fn configure_decoder_command(
    command: &mut Command,
    source: &Path,
    start_ms: u64,
    end_ms: u64,
    rate: f32,
) {
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-ss")
        .arg(format!("{:.3}", start_ms as f64 / 1_000.0))
        .arg("-readrate")
        .arg(format!("{:.2}", rate.clamp(0.5, 10.0)))
        .arg("-i")
        .arg(source)
        .arg("-t")
        .arg(format!(
            "{:.3}",
            end_ms.saturating_sub(start_ms).max(1) as f64 / 1_000.0
        ))
        .arg("-an")
        .arg("-sn")
        .arg("-vf")
        .arg("fps=15,scale=960:540:force_original_aspect_ratio=decrease,pad=960:540:(ow-iw)/2:(oh-ih)/2:color=black")
        .arg("-pix_fmt")
        .arg("rgba")
        .arg("-f")
        .arg("rawvideo")
        .arg("pipe:1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    hide_console(command);
}

fn hide_console(command: &mut Command) {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    #[cfg(not(windows))]
    let _ = command;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_outputs_silent_rgba_frames_to_a_pipe() {
        let mut command = Command::new("ffmpeg");
        configure_decoder_command(&mut command, Path::new("lesson.mp4"), 250, 2_250, 1.25);
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(args.windows(2).any(|pair| pair == ["-an", "-sn"]));
        assert!(args.windows(2).any(|pair| pair == ["-f", "rawvideo"]));
        assert!(args.iter().any(|arg| arg == "pipe:1"));
        assert!(!args.iter().any(|arg| arg == "-window_title"));
    }
}
