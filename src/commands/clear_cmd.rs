use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::{providers, state};

pub fn run() -> Result<()> {
    println!("This will remove AGENTS.md and every file dotai generated in:");
    println!("  - .claude/");
    println!("  - .cursor/");
    println!("  - .opencode/");
    println!("The .ai/ directory and files you created manually are kept.");
    print!("Are you sure you want to remove these files? [y/N] ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_ascii_lowercase();
    if answer != "y" && answer != "yes" {
        println!("Aborted.");
        return Ok(());
    }

    let ai_dir = Path::new(".ai");
    let mut manifest = state::Manifest::load(ai_dir)?;
    let mut removed = 0;
    if Path::new("AGENTS.md").exists() && manifest.owns("AGENTS.md") {
        fs::remove_file("AGENTS.md").with_context(|| "Failed to remove AGENTS.md")?;
        manifest.mark_removed("AGENTS.md");
        println!("Removed AGENTS.md");
        removed += 1;
    }
    for provider in providers::all() {
        removed += provider.cleanup(&mut manifest)?;
    }
    manifest.save(ai_dir)?;
    println!("Removed {} generated file(s).", removed);
    Ok(())
}
