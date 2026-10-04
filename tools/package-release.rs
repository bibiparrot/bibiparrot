//! Native release packaging. All orchestration, configuration and checks use Rust.
use std::{
    collections::HashMap,
    env,
    error::Error,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn argument(value: impl AsRef<OsStr>) -> OsString {
    value.as_ref().to_owned()
}
fn run(program: impl AsRef<OsStr>, arguments: Vec<OsString>) -> Result<()> {
    let status = Command::new(&program).args(arguments).status()?;
    if !status.success() {
        return Err(format!("{} failed: {status}", program.as_ref().to_string_lossy()).into());
    }
    Ok(())
}
macro_rules! cmd { ($program:expr $(, $arg:expr)* $(,)?) => { run($program, vec![$(argument($arg)),*]) }; }
fn output(program: &str, arguments: &[&OsStr]) -> Result<String> {
    let result = Command::new(program).args(arguments).output()?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned().into());
    }
    Ok(String::from_utf8(result.stdout)?)
}
fn tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
fn download(url: &str, path: &Path) -> Result<()> {
    cmd!("curl", "-fL", "--retry", "3", url, "-o", path)
}
fn executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
fn which(name: &str) -> Result<PathBuf> {
    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths)
                .map(|path| path.join(name))
                .find(|path| path.is_file())
        })
        .ok_or_else(|| format!("Missing tool: {name}").into())
}
fn resources(destination: &Path, ffmpeg: &str, gpl: bool) -> Result<()> {
    fs::create_dir_all(destination)?;
    let models = env::var_os("BIBIPARROT_MODEL_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("thirdparty/models"));
    tree(&models, &destination.join("thirdparty/models"))?;
    let mut config: toml::Value = toml::from_str(&fs::read_to_string("bibiparrot.toml")?)?;
    config["tools"]["ffmpeg"] = toml::Value::String(ffmpeg.into());
    fs::write(
        destination.join("bibiparrot.toml"),
        toml::to_string_pretty(&config)?,
    )?;
    for file in ["README.md", "THIRD_PARTY_NOTICES.md"] {
        fs::copy(file, destination.join(file))?;
    }
    let licenses = destination.join("licenses");
    fs::create_dir_all(&licenses)?;
    for (name, url) in [
        (
            "Whisper-MIT.txt",
            "https://raw.githubusercontent.com/openai/whisper/main/LICENSE",
        ),
        (
            "Silero-VAD-MIT.txt",
            "https://raw.githubusercontent.com/snakers4/silero-vad/master/LICENSE",
        ),
    ] {
        download(url, &licenses.join(name))?;
    }
    if gpl {
        for name in ["COPYING.GPLv2", "COPYING.GPLv3"] {
            download(
                &format!("https://raw.githubusercontent.com/FFmpeg/FFmpeg/master/{name}"),
                &licenses.join(name),
            )?;
        }
    }
    Ok(())
}

fn windows(version: &str, work: &Path, dist: &Path) -> Result<()> {
    let stage = work.join("BibiParrot");
    let sdk =
        PathBuf::from(env::var_os("FFMPEG_DIR").ok_or("FFMPEG_DIR must point to the FFmpeg SDK")?);
    resources(&stage, "ffmpeg.exe", false)?;
    fs::copy(
        "target/release/bibiparrot.exe",
        stage.join("bibiparrot.exe"),
    )?;
    for entry in fs::read_dir(sdk.join("bin"))? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "dll" || extension == "exe")
        {
            fs::copy(entry.path(), stage.join(entry.file_name()))?;
        }
    }
    fs::copy(sdk.join("LICENSE.txt"), stage.join("FFmpeg-LICENSE.txt"))?;
    let tar = PathBuf::from(env::var_os("WINDIR").unwrap_or_else(|| "C:/Windows".into()))
        .join("System32/tar.exe");
    cmd!(
        tar,
        "-a",
        "-cf",
        dist.join(format!("BibiParrot-{version}-windows-x86_64.zip")),
        "-C",
        work,
        "BibiParrot"
    )
}

fn linux(version: &str, arch: &str, work: &Path, dist: &Path) -> Result<()> {
    let appdir = work.join("AppDir");
    let binaries = appdir.join("usr/bin");
    resources(&binaries, "ffmpeg", true)?;
    fs::copy("target/release/bibiparrot", binaries.join("bibiparrot"))?;
    let desktop = work.join("bibiparrot.desktop");
    fs::write(&desktop, "[Desktop Entry]\nType=Application\nName=BibiParrot\nComment=Listen, type, repeat\nExec=bibiparrot\nIcon=bibi-icon\nCategories=Education;AudioVideo;\nTerminal=false\n")?;
    let deploy = work.join(format!("linuxdeploy-{arch}.AppImage"));
    download(&format!("https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-{arch}.AppImage"), &deploy)?;
    executable(&deploy)?;
    let status = Command::new(&deploy)
        .env("APPIMAGE_EXTRACT_AND_RUN", "1")
        .env("ARCH", arch)
        .env(
            "OUTPUT",
            dist.join(format!("BibiParrot-{version}-linux-{arch}.AppImage")),
        )
        .arg("--appdir")
        .arg(&appdir)
        .arg("--executable")
        .arg(binaries.join("bibiparrot"))
        .arg("--executable")
        .arg(which("ffmpeg")?)
        .arg("--executable")
        .arg(which("ffplay")?)
        .arg("--executable")
        .arg(which("ffprobe")?)
        .arg("--desktop-file")
        .arg(&desktop)
        .args([
            "--icon-file",
            "assets/bibi-icon.png",
            "--output",
            "appimage",
        ])
        .status()?;
    if !status.success() {
        return Err("linuxdeploy failed".into());
    }
    let bundle = work.join("BibiParrot");
    tree(&binaries, &bundle.join("bin"))?;
    tree(&appdir.join("usr/lib"), &bundle.join("lib"))?;
    let launcher = bundle.join("BibiParrot");
    fs::write(&launcher, "#!/bin/sh\nset -eu\nAPP_DIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\nexport LD_LIBRARY_PATH=\"$APP_DIR/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}\"\nexport PATH=\"$APP_DIR/bin:$PATH\"\nexec \"$APP_DIR/bin/bibiparrot\" \"$@\"\n")?;
    executable(&launcher)?;
    cmd!(
        "tar",
        "-czf",
        dist.join(format!("BibiParrot-{version}-linux-{arch}.tar.gz")),
        "-C",
        work,
        "BibiParrot"
    )?;
    let rpmroot = work.join("rpmroot");
    tree(&bundle, &rpmroot.join("opt/bibiparrot"))?;
    fs::create_dir_all(rpmroot.join("usr/bin"))?;
    let wrapper = rpmroot.join("usr/bin/bibiparrot");
    fs::write(
        &wrapper,
        "#!/bin/sh\nexec /opt/bibiparrot/BibiParrot \"$@\"\n",
    )?;
    executable(&wrapper)?;
    fs::create_dir_all(rpmroot.join("usr/share/applications"))?;
    fs::copy(
        &desktop,
        rpmroot.join("usr/share/applications/bibiparrot.desktop"),
    )?;
    let icons = rpmroot.join("usr/share/icons/hicolor/256x256/apps");
    fs::create_dir_all(&icons)?;
    fs::copy("assets/bibi-icon.png", icons.join("bibi-icon.png"))?;
    cmd!(
        "fpm",
        "-s",
        "dir",
        "-t",
        "rpm",
        "-n",
        "bibiparrot",
        "-v",
        version,
        "--architecture",
        arch,
        "--rpm-auto-add-directories",
        "--depends",
        "glibc >= 2.39",
        "--description",
        "Offline listening and dictation workspace",
        "--url",
        "https://github.com/bibiparrot/bibiparrot",
        "-C",
        &rpmroot,
        "-p",
        dist.join(format!("BibiParrot-{version}-linux-{arch}.rpm")),
        "."
    )?;
    let library_path = bundle.join("lib");
    for binary in ["bibiparrot", "ffmpeg", "ffplay", "ffprobe"] {
        let result = Command::new("ldd")
            .arg(bundle.join("bin").join(binary))
            .env("LD_LIBRARY_PATH", &library_path)
            .output()?;
        if !result.status.success() || String::from_utf8_lossy(&result.stdout).contains("not found")
        {
            return Err(format!("Unresolved Linux media dependency: {binary}").into());
        }
    }
    Ok(())
}

fn patch_dylibs(
    binary: &Path,
    original: &Path,
    frameworks: &Path,
    copied: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    let dependencies = output("otool", &[OsStr::new("-L"), original.as_os_str()])?;
    for line in dependencies.lines().skip(1) {
        let name = line
            .trim()
            .split(" (")
            .next()
            .ok_or("Invalid otool dependency")?;
        if name.starts_with("/usr/lib/") || name.starts_with("/System/") {
            continue;
        }
        let parent = original
            .parent()
            .ok_or("Missing library parent")?
            .to_string_lossy();
        let mut dependency = PathBuf::from(name.replace("@loader_path", &parent));
        if let Some(relative) = name.strip_prefix("@rpath/") {
            let commands = output("otool", &[OsStr::new("-l"), original.as_os_str()])?;
            let mut candidates: Vec<_> = commands
                .lines()
                .filter_map(|line| {
                    line.trim()
                        .strip_prefix("path ")
                        .and_then(|line| line.split(" (offset").next())
                })
                .map(|path| {
                    PathBuf::from(
                        path.replace("@loader_path", &parent)
                            .replace("@executable_path", &parent),
                    )
                    .join(relative)
                })
                .collect();
            candidates.push(
                PathBuf::from(output("brew", &[OsStr::new("--prefix")])?.trim())
                    .join("lib")
                    .join(relative),
            );
            dependency = candidates
                .into_iter()
                .find(|path| path.is_file())
                .ok_or_else(|| format!("Unresolved macOS dependency: {name}"))?;
        }
        let filename = dependency
            .file_name()
            .ok_or("Library has no name")?
            .to_string_lossy()
            .into_owned();
        let resolved = dependency.canonicalize()?;
        let target = frameworks.join(&filename);
        if let Some(existing) = copied.get(&filename) {
            if *existing != resolved {
                return Err(format!("Conflicting library: {filename}").into());
            }
        } else {
            copied.insert(filename.clone(), resolved.clone());
            fs::copy(&resolved, &target)?;
            executable(&target)?;
            cmd!(
                "install_name_tool",
                "-id",
                format!("@rpath/{filename}"),
                &target
            )?;
            patch_dylibs(&target, &resolved, frameworks, copied)?;
        }
        cmd!(
            "install_name_tool",
            "-change",
            name,
            format!("@rpath/{filename}"),
            binary
        )?;
    }
    let rpath = if binary.parent() == Some(frameworks) {
        "@loader_path"
    } else {
        "@executable_path/../Frameworks"
    };
    if !output("otool", &[OsStr::new("-l"), binary.as_os_str()])?
        .contains(&format!("path {rpath} "))
    {
        cmd!("install_name_tool", "-add_rpath", rpath, binary)?;
    }
    Ok(())
}

fn macos(version: &str, arch: &str, work: &Path, dist: &Path) -> Result<()> {
    let app = work.join("BibiParrot.app");
    let contents = app.join("Contents");
    let binaries = contents.join("MacOS");
    resources(&binaries, "ffmpeg", true)?;
    fs::copy("target/release/bibiparrot", binaries.join("bibiparrot"))?;
    for name in ["ffmpeg", "ffplay", "ffprobe"] {
        fs::copy(which(name)?.canonicalize()?, binaries.join(name))?;
    }
    let resources_dir = contents.join("Resources");
    fs::create_dir_all(&resources_dir)?;
    let iconset = work.join("bibi.iconset");
    fs::create_dir_all(&iconset)?;
    for size in [16, 32, 128, 256, 512] {
        for scale in [1, 2] {
            let filename = format!(
                "icon_{size}x{size}{}.png",
                if scale == 2 { "@2x" } else { "" }
            );
            cmd!(
                "sips",
                "-z",
                (size * scale).to_string(),
                (size * scale).to_string(),
                "assets/bibi-icon.png",
                "--out",
                iconset.join(filename)
            )?;
        }
    }
    cmd!(
        "iconutil",
        "-c",
        "icns",
        &iconset,
        "-o",
        resources_dir.join("bibi.icns")
    )?;
    fs::write(
        contents.join("Info.plist"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleName</key><string>BibiParrot</string><key>CFBundleIdentifier</key><string>com.bibiparrot.egui</string><key>CFBundleExecutable</key><string>bibiparrot</string><key>CFBundleVersion</key><string>{version}</string><key>CFBundleShortVersionString</key><string>{version}</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleIconFile</key><string>bibi.icns</string><key>LSMinimumSystemVersion</key><string>11.0</string><key>NSHighResolutionCapable</key><true/></dict></plist>"#
        ),
    )?;
    let frameworks = contents.join("Frameworks");
    fs::create_dir_all(&frameworks)?;
    let mut copied = HashMap::new();
    for name in ["bibiparrot", "ffmpeg", "ffplay", "ffprobe"] {
        let binary = binaries.join(name);
        patch_dylibs(&binary, &binary, &frameworks, &mut copied)?;
    }
    cmd!("codesign", "--force", "--deep", "--sign", "-", &app)?;
    cmd!("codesign", "--verify", "--deep", "--strict", &app)?;
    cmd!(
        "ditto",
        "-c",
        "-k",
        "--sequesterRsrc",
        "--keepParent",
        &app,
        dist.join(format!("BibiParrot-{version}-{arch}-macOS.zip"))
    )?;
    let dmg = work.join("dmg");
    fs::create_dir_all(&dmg)?;
    cmd!("ditto", &app, dmg.join("BibiParrot.app"))?;
    #[cfg(unix)]
    std::os::unix::fs::symlink("/Applications", dmg.join("Applications"))?;
    cmd!(
        "hdiutil",
        "create",
        "-volname",
        "BibiParrot",
        "-srcfolder",
        &dmg,
        "-format",
        "UDZO",
        dist.join(format!("BibiParrot-{version}-{arch}-macOS.dmg"))
    )?;
    cmd!(
        "pkgbuild",
        "--component",
        &app,
        "--install-location",
        "/Applications",
        "--identifier",
        "com.bibiparrot.egui",
        "--version",
        version,
        dist.join(format!("BibiParrot-{version}-{arch}-macOS.pkg"))
    )
}

fn main() -> Result<()> {
    env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    let args: Vec<_> = env::args().collect();
    let platform = args
        .get(1)
        .ok_or("Usage: package-release <windows|linux|macos> <arch>")?;
    let arch = args.get(2).ok_or("Missing architecture")?;
    let manifest: toml::Value = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    let version = manifest["package"]["version"]
        .as_str()
        .ok_or("Missing package version")?;
    let root = env::current_dir()?;
    let work = root.join("package-work");
    let dist = root.join("dist");
    fs::create_dir_all(&work)?;
    fs::create_dir_all(&dist)?;
    match platform.as_str() {
        "windows" => windows(version, &work, &dist),
        "linux" => linux(version, arch, &work, &dist),
        "macos" => macos(version, arch, &work, &dist),
        _ => Err("Unsupported release platform".into()),
    }
}
