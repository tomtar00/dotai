use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::{ai_dir, config, providers};

pub fn run(providers: Vec<String>) -> Result<()> {
    let ai_dir_path = Path::new(".ai");
    if !ai_dir_path.exists() {
        anyhow::bail!(".ai directory does not exist. Run `dotai new` first");
    }
    if !ai_dir_path.join("AI.md").exists() {
        anyhow::bail!(".ai/AI.md not found");
    }

    let project = ai_dir::load(ai_dir_path)?;

    let selected = if providers.is_empty() {
        match config::load()? {
            Some(cfg) => cfg.providers,
            None => {
                anyhow::bail!(
                    "No {} found. Create it with a \"providers\" list, or pass them: `dotai gen claude opencode cursor`",
                    config::path().display()
                )
            }
        }
    } else {
        providers
    };

    let resolved = providers::resolve(&selected)?;

    if let Some(ai_md) = &project.ai_md {
        let mut content = ai_md.clone();
        if !content.ends_with('\n') {
            content.push('\n');
        }
        fs::write("AGENTS.md", content).with_context(|| "Failed to write AGENTS.md")?;
        println!("Wrote AGENTS.md");
    }
    for provider in &resolved {
        provider
            .generate(&project, ai_dir_path)
            .with_context(|| format!("Failed to generate {} config", provider.name()))?;
    }

    let names: Vec<String> = resolved.iter().map(|p| p.name().to_string()).collect();
    println!("Done. Generated: {}", names.join(", "));
    Ok(())
}
