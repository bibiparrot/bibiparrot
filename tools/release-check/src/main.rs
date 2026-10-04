fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args_os().nth(1).ok_or("Usage: release-check <directory>")?;
    release_check::verify(std::path::Path::new(&directory))
}
