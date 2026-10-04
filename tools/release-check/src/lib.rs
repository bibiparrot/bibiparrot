use sha2::{Digest, Sha256};
use std::{error::Error, fs::{self, File}, io::{Read, Write}, path::Path};

pub fn verify(directory: &Path) -> Result<(), Box<dyn Error>> {
    let suffixes = [
        "-arm64-macOS.dmg", "-arm64-macOS.pkg", "-arm64-macOS.zip",
        "-x86_64-macOS.dmg", "-x86_64-macOS.pkg", "-x86_64-macOS.zip",
        "-linux-aarch64.AppImage", "-linux-aarch64.rpm", "-linux-aarch64.tar.gz",
        "-linux-x86_64.AppImage", "-linux-x86_64.rpm", "-linux-x86_64.tar.gz",
        "-windows-x86_64.zip",
    ];
    let mut files = fs::read_dir(directory)?.map(|entry| entry.map(|entry| entry.path())).collect::<Result<Vec<_>, _>>()?;
    files.sort();
    for suffix in suffixes {
        let matches: Vec<_> = files.iter().filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(suffix))).collect();
        if matches.len() != 1 || fs::metadata(matches[0])?.len() == 0 {
            return Err(format!("Missing or ambiguous release package: {suffix}").into());
        }
    }
    let mut output = File::create(directory.join("SHA256SUMS.txt"))?;
    let mut buffer = vec![0_u8; 1024 * 1024];
    for path in files {
        if !path.is_file() || path.file_name().is_some_and(|name| name == "SHA256SUMS.txt") { continue; }
        let mut file = File::open(&path)?;
        let mut hash = Sha256::new();
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 { break; }
            hash.update(&buffer[..count]);
        }
        writeln!(output, "{:x}  {}", hash.finalize(), path.file_name().unwrap().to_string_lossy())?;
    }
    println!("Verified all 13 desktop packages.");
    Ok(())
}
