# Building BibiParrot

Use stable Rust, Clang/libclang, CMake and the FFmpeg development libraries. The release workflow builds on native Windows, Linux x86_64/aarch64 and macOS Intel/Apple Silicon runners.

On Ubuntu 24.04 install `clang cmake pkg-config libavcodec-dev libavformat-dev libavutil-dev libswresample-dev libgtk-3-dev libx11-dev libxi-dev libxrandr-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev ffmpeg`. On macOS install `ffmpeg pkg-config` with Homebrew. On Windows point `FFMPEG_DIR` at a shared FFmpeg SDK and `LIBCLANG_PATH` at the LLVM bin directory; build from an MSVC developer shell.

```sh
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo build --release --locked
```

The developer-only `.cargo/config.toml` is not included in the published repository because its paths refer to the original workstation.

The default analysis path decodes to 16 kHz mono PCM once, computes 10 ms RMS windows, estimates the noise floor, and splits at sustained low energy. It preserves padding, caps long spans, and shares one Whisper model between independently owned worker states. Results return to the UI in chronological order; punctuation refines the final sentence tree. No intermediate sentence audio files or whole-file recognition pass are required.

Tune `[segmentation]` in `bibiparrot.toml`: `silence_relative_db`, optional `silence_threshold_db` (dBFS), `silence_gap_ms`, `min_speech_ms`, `speech_padding_ms`, `max_sentence_ms`, `whisper_workers`, and `whisper_threads`. Automatic thread allocation caps total CPU threads at eight. Set `use_volume_segmentation = false` to use the previous Whisper/VAD pipeline. Threshold boundaries describe pauses; they are not a guarantee of grammatical sentence boundaries.

Tagged builds publish only after all five platform jobs produce all 13 requested packages. The Rust `release-check` tool verifies the artifact set and writes SHA-256 checksums. The Rust `package-release` tool includes speech models and media tools. macOS packages are ad-hoc signed without notarization.

Screenshot fixtures are prepared with `cargo run --bin screenshot-fixture -- <local media repository>`. Capture uses the application's existing `BIBIPARROT_QA_SCREENSHOT`, `BIBIPARROT_QA_DATA_DIR`, `BIBIPARROT_QA_LESSON`, and optional `BIBIPARROT_QA_COLLAPSED_DOCKS` environment settings. Fixtures are isolated from the user's library. No Python runtime is used.
