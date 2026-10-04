use crate::{config::LoadedConfig, model::MediaKind};
use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, ChildStderr, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Instant,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendKind {
    Mpv,
    Ffplay,
    Unavailable,
}

pub struct ProcessPlayer {
    backend: BackendKind,
    executable: Option<PathBuf>,
    child: Option<Child>,
    playing: bool,
    range_start_ms: u64,
    range_end_ms: u64,
    cursor_ms: u64,
    started_at: Option<Instant>,
    rate: f32,
    volume: u8,
    looping: bool,
    path: PathBuf,
    ffplay_clock: Option<Receiver<u64>>,
    spawn_start_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct PlaybackSettings {
    pub looping: bool,
    pub rate: f32,
    pub volume: u8,
}

impl ProcessPlayer {
    pub fn new(config: &LoadedConfig) -> Self {
        let mpv = config
            .mpv()
            .or_else(|| {
                env::var_os("BIBIPARROT_MPV")
                    .map(PathBuf::from)
                    .filter(|path| path.exists())
            })
            .or_else(|| resolve_binary("mpv.exe"));
        let (backend, executable) = if let Some(path) = mpv {
            (BackendKind::Mpv, Some(path))
        } else if let Some(path) = resolve_binary("ffplay.exe") {
            (BackendKind::Ffplay, Some(path))
        } else {
            (BackendKind::Unavailable, None)
        };
        Self {
            backend,
            executable,
            child: None,
            playing: false,
            range_start_ms: 0,
            range_end_ms: 1,
            cursor_ms: 0,
            started_at: None,
            rate: 1.0,
            volume: config.values.playback.default_volume.min(100),
            looping: false,
            path: PathBuf::new(),
            ffplay_clock: None,
            spawn_start_ms: 0,
        }
    }

    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            BackendKind::Mpv => "mpv",
            BackendKind::Ffplay => "FFplay",
            BackendKind::Unavailable => "no media backend",
        }
    }

    pub fn is_available(&self) -> bool {
        self.backend != BackendKind::Unavailable
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn position_ms(&self) -> u64 {
        if self.playing && self.backend != BackendKind::Ffplay {
            let elapsed = self
                .started_at
                .map(|started| started.elapsed().as_secs_f64())
                .unwrap_or_default();
            let position = self.cursor_ms as f64 + elapsed * 1000.0 * self.rate as f64;
            (position as u64).min(self.range_end_ms)
        } else {
            self.cursor_ms
        }
    }

    pub fn range(&self) -> (u64, u64) {
        (self.range_start_ms, self.range_end_ms)
    }

    pub fn start(
        &mut self,
        path: &Path,
        _kind: MediaKind,
        start_ms: u64,
        end_ms: u64,
        settings: PlaybackSettings,
    ) -> Result<(), String> {
        self.stop_child();
        if !self.is_available() {
            return Err("No mpv.exe or ffplay.exe media backend was found".into());
        }
        let end_ms = end_ms.max(start_ms + 1);
        self.path = path.to_path_buf();
        self.range_start_ms = start_ms;
        self.range_end_ms = end_ms;
        self.cursor_ms = start_ms;
        self.looping = settings.looping;
        self.rate = settings.rate.clamp(0.5, 2.0);
        self.volume = settings.volume.min(100);
        self.spawn_from(start_ms)?;
        self.playing = true;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    pub fn pause(&mut self) {
        self.cursor_ms = self.position_ms();
        self.playing = false;
        self.started_at = None;
        self.stop_child();
    }

    pub fn resume(&mut self) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            return Err("No media has been selected".into());
        }
        if self.cursor_ms >= self.range_end_ms {
            self.cursor_ms = self.range_start_ms;
        }
        self.spawn_from(self.cursor_ms)?;
        self.playing = true;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    pub fn seek(&mut self, position_ms: u64) -> Result<(), String> {
        self.cursor_ms = position_ms.clamp(self.range_start_ms, self.range_end_ms);
        if self.playing {
            self.stop_child();
            self.spawn_from(self.cursor_ms)?;
            self.started_at = Some(Instant::now());
        }
        Ok(())
    }

    pub fn set_volume(&mut self, volume: u8) -> Result<(), String> {
        self.volume = volume.min(100);
        if self.playing {
            self.cursor_ms = self.position_ms();
            self.stop_child();
            self.spawn_from(self.cursor_ms)?;
            self.started_at = Some(Instant::now());
        }
        Ok(())
    }

    pub fn set_rate(&mut self, rate: f32) -> Result<(), String> {
        let rate = rate.clamp(0.5, 2.0);
        if (self.rate - rate).abs() < f32::EPSILON {
            return Ok(());
        }
        if self.playing {
            self.cursor_ms = self.position_ms();
            self.stop_child();
        }
        self.rate = rate;
        if self.playing {
            self.spawn_from(self.cursor_ms)?;
            self.started_at = Some(Instant::now());
        }
        Ok(())
    }

    /// Updates repetition without interrupting the current pass. Disabling
    /// looping therefore lets playback continue to `range_end_ms` and stop.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    pub fn tick(&mut self) -> Result<(), String> {
        self.update_ffplay_clock();
        let finished = match self.child.as_mut() {
            Some(child) => child
                .try_wait()
                .map_err(|error| format!("Could not query media backend: {error}"))?
                .is_some(),
            None => true,
        };
        if !self.playing || !finished {
            return Ok(());
        }
        if self.looping {
            self.stop_child();
            self.cursor_ms = self.range_start_ms;
            self.spawn_from(self.cursor_ms)?;
            self.started_at = Some(Instant::now());
        } else {
            self.cursor_ms = self.range_end_ms;
            self.playing = false;
            self.started_at = None;
            self.stop_child();
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.cursor_ms = self.range_start_ms;
        self.started_at = None;
        self.stop_child();
    }

    fn spawn_from(&mut self, start_ms: u64) -> Result<(), String> {
        let executable = self
            .executable
            .as_ref()
            .ok_or("Media backend unavailable")?;
        let remaining_ms = self.range_end_ms.saturating_sub(start_ms).max(1);
        let mut command = Command::new(executable);
        match self.backend {
            BackendKind::Mpv => configure_mpv_command(
                &mut command,
                &self.path,
                start_ms,
                self.range_end_ms,
                self.rate,
                self.volume,
            ),
            BackendKind::Ffplay => configure_ffplay_command(
                &mut command,
                &self.path,
                start_ms,
                remaining_ms,
                self.rate,
                self.volume,
            ),
            BackendKind::Unavailable => return Err("Media backend unavailable".into()),
        }
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(
            if self.backend == BackendKind::Ffplay {
                Stdio::piped()
            } else {
                Stdio::null()
            },
        );
        hide_console(&mut command);
        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not start media backend: {error}"))?;
        if let Some(stderr) = child.stderr.take() {
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || read_ffplay_clock(stderr, sender));
            self.ffplay_clock = Some(receiver);
        }
        self.child = Some(child);
        self.spawn_start_ms = start_ms;
        Ok(())
    }

    fn update_ffplay_clock(&mut self) {
        let Some(receiver) = &self.ffplay_clock else {
            return;
        };
        while let Ok(clock_ms) = receiver.try_recv() {
            self.cursor_ms =
                ffplay_position_ms(self.spawn_start_ms, self.range_end_ms, clock_ms, self.rate);
        }
    }

    fn stop_child(&mut self) {
        self.ffplay_clock = None;
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn configure_mpv_command(
    command: &mut Command,
    path: &Path,
    start_ms: u64,
    end_ms: u64,
    rate: f32,
    volume: u8,
) {
    command
        .arg("--no-terminal")
        .arg("--really-quiet")
        .arg("--force-window=no")
        .arg("--vid=no")
        .arg(format!("--start={:.3}", start_ms as f64 / 1_000.0))
        .arg(format!("--end={:.3}", end_ms as f64 / 1_000.0))
        .arg(format!("--speed={:.2}", rate))
        .arg(format!("--volume={volume}"))
        .arg(path);
}

fn configure_ffplay_command(
    command: &mut Command,
    path: &Path,
    start_ms: u64,
    remaining_ms: u64,
    rate: f32,
    volume: u8,
) {
    // Decode preroll to restore the MP3 bit reservoir before trimming the word.
    // A direct packet seek can otherwise turn the first phoneme into silence.
    let seek_ms = start_ms.saturating_sub(1_000);
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-stats")
        .arg("-autoexit")
        .arg("-nodisp")
        .arg("-vn")
        .arg("-ss")
        .arg(format!("{:.3}", seek_ms as f64 / 1_000.0))
        .arg("-t")
        .arg(format!(
            "{:.3}",
            (remaining_ms + start_ms - seek_ms) as f64 / 1_000.0
        ))
        .arg("-volume")
        .arg(volume.to_string())
        .arg("-af")
        .arg(format!(
            "atrim=start={:.3}:end={:.3},atempo={:.2}",
            start_ms as f64 / 1_000.0,
            start_ms.saturating_add(remaining_ms) as f64 / 1_000.0,
            rate,
        ))
        .arg(path);
}

fn read_ffplay_clock(stderr: ChildStderr, sender: mpsc::Sender<u64>) {
    for line in BufReader::new(stderr).split(b'\r').flatten() {
        let line = String::from_utf8_lossy(&line);
        if let Some(clock_ms) = parse_ffplay_clock(&line) {
            if sender.send(clock_ms).is_err() {
                break;
            }
        }
    }
}

fn parse_ffplay_clock(line: &str) -> Option<u64> {
    let seconds: f64 = line.split_whitespace().next()?.parse().ok()?;
    seconds
        .is_finite()
        .then(|| (seconds.max(0.0) * 1_000.0).round() as u64)
}

fn ffplay_position_ms(start_ms: u64, end_ms: u64, clock_ms: u64, rate: f32) -> u64 {
    (start_ms as f64 + clock_ms.saturating_sub(start_ms) as f64 * rate as f64)
        .round()
        .min(end_ms as f64) as u64
}

impl Drop for ProcessPlayer {
    fn drop(&mut self) {
        self.stop_child();
    }
}

pub fn probe_duration(path: &Path, config: &LoadedConfig) -> Option<u64> {
    if let Some(ffprobe) = config
        .ffprobe()
        .or_else(|| resolve_binary(binary_name("ffprobe")))
    {
        return probe_with_ffprobe(path, &ffprobe);
    }
    let ffmpeg = config
        .ffmpeg()
        .or_else(|| resolve_binary(binary_name("ffmpeg")))?;
    probe_with_ffmpeg(path, &ffmpeg)
}

fn probe_with_ffprobe(path: &Path, ffprobe: &Path) -> Option<u64> {
    let mut command = Command::new(ffprobe);
    command
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(path)
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    hide_console(&mut command);
    let output = command.output().ok()?;
    let seconds: f64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;
    Some((seconds * 1000.0).round().max(1.0) as u64)
}

fn probe_with_ffmpeg(path: &Path, ffmpeg: &Path) -> Option<u64> {
    let mut command = Command::new(ffmpeg);
    command
        .arg("-hide_banner")
        .arg("-i")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    hide_console(&mut command);
    let output = command.output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let duration = stderr
        .lines()
        .find_map(|line| line.split_once("Duration:").map(|(_, rest)| rest.trim()))?;
    let timestamp = duration.split(',').next()?.trim();
    parse_timestamp(timestamp)
}

fn parse_timestamp(timestamp: &str) -> Option<u64> {
    let mut parts = timestamp.split(':');
    let hours: f64 = parts.next()?.parse().ok()?;
    let minutes: f64 = parts.next()?.parse().ok()?;
    let seconds: f64 = parts.next()?.parse().ok()?;
    Some(((hours * 3_600.0 + minutes * 60.0 + seconds) * 1_000.0).round() as u64)
}

fn binary_name(stem: &str) -> &str {
    #[cfg(windows)]
    {
        if stem == "ffprobe" {
            "ffprobe.exe"
        } else {
            "ffmpeg.exe"
        }
    }
    #[cfg(not(windows))]
    {
        stem
    }
}

fn resolve_binary(name: &str) -> Option<PathBuf> {
    #[cfg(not(windows))]
    let name = name.strip_suffix(".exe").unwrap_or(name);
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|candidate| candidate.exists())
    })
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
    fn mpv_backend_never_creates_a_video_window() {
        let mut command = Command::new("mpv");
        configure_mpv_command(&mut command, Path::new("lesson.mp4"), 0, 1_000, 1.0, 80);
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(args.iter().any(|arg| arg == "--force-window=no"));
        assert!(args.iter().any(|arg| arg == "--vid=no"));
        assert!(!args
            .iter()
            .any(|arg| arg.contains("force-window=immediate")));
    }

    #[test]
    fn ffplay_backend_is_audio_only_and_has_no_window_title() {
        let mut command = Command::new("ffplay");
        configure_ffplay_command(&mut command, Path::new("lesson.mp4"), 0, 1_000, 1.0, 80);
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(args.iter().any(|arg| arg == "-nodisp"));
        assert!(args.iter().any(|arg| arg == "-vn"));
        assert!(!args.iter().any(|arg| arg == "-window_title"));
    }

    #[test]
    fn disabling_loop_during_playback_stops_at_the_current_range_end() {
        let mut player = ProcessPlayer {
            backend: BackendKind::Unavailable,
            executable: None,
            child: None,
            playing: true,
            range_start_ms: 100,
            range_end_ms: 200,
            cursor_ms: 100,
            started_at: Some(Instant::now() - std::time::Duration::from_secs(1)),
            rate: 1.0,
            volume: 80,
            looping: true,
            path: PathBuf::new(),
            ffplay_clock: None,
            spawn_start_ms: 100,
        };

        player.set_looping(false);
        player.tick().unwrap();

        assert!(!player.is_playing());
        assert_eq!(player.position_ms(), 200);
    }

    #[test]
    fn ffplay_clock_uses_the_audio_master_timestamp() {
        assert_eq!(parse_ffplay_clock("   1.42 M-A:  0.000 fd=0"), Some(1_420));
        assert_eq!(parse_ffplay_clock("    nan M-A:    nan fd=0"), None);
        assert_eq!(ffplay_position_ms(1_000, 2_000, 1_420, 2.0), 1_840);
    }

    #[test]
    fn word_playback_decodes_preroll_but_trims_to_only_the_selected_word() {
        let mut command = Command::new("ffplay");
        configure_ffplay_command(
            &mut command,
            Path::new("first_snowfall.mp3"),
            660,
            760,
            1.0,
            80,
        );
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect();
        assert!(args.windows(2).any(|pair| pair == ["-ss", "0.000"]));
        assert!(args.windows(2).any(|pair| pair == ["-t", "1.420"]));
        assert!(args
            .iter()
            .any(|arg| arg == "atrim=start=0.660:end=1.420,atempo=1.00"));
    }

    #[test]
    #[ignore = "requires local ffplay and first_snowfall.mp3"]
    fn ffplay_clock_tracks_real_short_audio_until_the_process_finishes() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../first_snowfall.mp3");
        let mut player = ProcessPlayer::new(&LoadedConfig::load());
        player
            .start(
                &source,
                MediaKind::Audio,
                0,
                2_299,
                PlaybackSettings {
                    looping: false,
                    rate: 1.0,
                    volume: 0,
                },
            )
            .unwrap();
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        let mut saw_progress = false;
        while player.is_playing() && Instant::now() < deadline {
            thread::sleep(std::time::Duration::from_millis(40));
            player.tick().unwrap();
            saw_progress |= player.position_ms() > 0 && player.position_ms() < 2_299;
        }
        assert!(saw_progress, "FFplay did not report its audio clock");
        assert!(!player.is_playing(), "FFplay did not finish before timeout");
        assert_eq!(player.position_ms(), 2_299);
    }
}
