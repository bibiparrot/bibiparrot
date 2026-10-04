fn main() {
    println!("cargo:rerun-if-changed=assets/bibi.ico");
    println!("cargo:rerun-if-changed=bibiparrot.toml");
    println!("cargo:rerun-if-changed=THIRD_PARTY_NOTICES.md");
    println!(
        "cargo:rerun-if-changed=../../thirdparty/ffmpeg-n9.0-latest-win64-lgpl-shared-9.0/bin"
    );
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        if let Some(profile_dir) = std::path::Path::new(&out_dir).ancestors().nth(3) {
            let _ = std::fs::copy("bibiparrot.toml", profile_dir.join("bibiparrot.toml"));
            let _ = std::fs::copy(
                "THIRD_PARTY_NOTICES.md",
                profile_dir.join("THIRD_PARTY_NOTICES.md"),
            );
            #[cfg(target_os = "windows")]
            {
                let ffmpeg_dir = std::env::var_os("FFMPEG_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| {
                        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join("../../thirdparty/ffmpeg-n9.0-latest-win64-lgpl-shared-9.0")
                    });
                let ffmpeg_bin = ffmpeg_dir.join("bin");
                if let Ok(entries) = std::fs::read_dir(ffmpeg_bin) {
                    for source in entries.flatten().map(|entry| entry.path()).filter(|path| {
                        path.extension().is_some_and(|extension| {
                            extension.to_string_lossy().eq_ignore_ascii_case("dll")
                        })
                    }) {
                        if let Some(name) = source.file_name() {
                            let _ = std::fs::copy(&source, profile_dir.join(name));
                            let _ = std::fs::copy(&source, profile_dir.join("deps").join(name));
                        }
                    }
                }
                let _ = std::fs::copy(
                    ffmpeg_dir.join("LICENSE.txt"),
                    profile_dir.join("FFmpeg-LICENSE.txt"),
                );
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let mut resource = winres::WindowsResource::new();
        resource
            .set_icon("assets/bibi.ico")
            .set("ProductName", "BibiParrot")
            .set(
                "FileDescription",
                "BibiParrot language listening and dictation",
            )
            .set("LegalCopyright", "BibiParrot contributors");
        resource.compile().expect("compile Windows resources");
    }
}
