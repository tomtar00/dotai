use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/dotai")
}

pub fn template_dir() -> PathBuf {
    config_dir().join("template")
}

pub fn copy_dir(src: &Path, dst: &Path, skip_dotfiles: bool) -> Result<()> {
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory: {}", dst.display()))?;

    for entry in
        fs::read_dir(src).with_context(|| format!("Failed to read directory: {}", src.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        if skip_dotfiles && name.to_string_lossy().starts_with('.') {
            continue;
        }
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_dir(&src.join(&name), &dst.join(&name), skip_dotfiles)?;
        } else if ft.is_file() {
            fs::copy(src.join(&name), dst.join(&name)).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    src.join(&name).display(),
                    dst.join(&name).display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))
}
