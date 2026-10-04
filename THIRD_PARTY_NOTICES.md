# Third-party notices

## Font Awesome Free icons

BibiParrot includes selected SVG icons from Font Awesome Free through the
`pictogram` and `pictogram-icons-font-awesome` Rust crates.

- Copyright (c) Fonticons, Inc. ([fontawesome.com](https://fontawesome.com))
- Icons are licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
- Source: [Font Awesome](https://github.com/FortAwesome/Font-Awesome)

## Media decoding and speech recognition

- Windows packages bundle BtbN's LGPL FFmpeg shared build (LGPL 2.1 or later).
- Linux and macOS packages include distribution/Homebrew FFmpeg builds, which enable GPL components. These FFmpeg binaries are GPL 2.0 or later; see their `ffmpeg -L` output and the accompanying FFmpeg license files. FFmpeg source and build instructions: https://ffmpeg.org/ and https://github.com/FFmpeg/FFmpeg.
- `whisper-rs` is Unlicense/public domain and statically builds whisper.cpp/ggml, licensed under MIT.

- Bundled Whisper models: OpenAI Whisper, MIT license, https://github.com/openai/whisper/blob/main/LICENSE.
- Bundled Silero VAD model: MIT license, https://github.com/snakers4/silero-vad/blob/master/LICENSE.
