use std::path::PathBuf;

pub fn downloads_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not find your home directory".to_string())?;
    Ok(PathBuf::from(home).join("Downloads"))
}