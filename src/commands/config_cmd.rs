use anyhow::{Context, Result};
use std::process::Command;

use crate::config;

pub fn run() -> Result<()> {
    config::ensure_default()?;
    let path = config::path();
    let editor = std::env::var("DOTAI_EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| default_editor().to_string());
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (program, args) = parts
        .split_first()
        .map(|(p, a)| (p.to_string(), a.to_vec()))
        .unwrap_or_else(|| (default_editor().to_string(), vec![]));
    let status = Command::new(&program)
        .args(args)
        .arg(&path)
        .status()
        .with_context(|| format!("Failed to run editor '{}'", editor))?;
    if !status.success() {
        anyhow::bail!("editor exited with {}", status);
    }
    Ok(())
}

fn default_editor() -> &'static str {
    if cfg!(windows) {
        "notepad"
    } else {
        "vi"
    }
}
