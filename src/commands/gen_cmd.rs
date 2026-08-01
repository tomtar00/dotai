use anyhow::{Context, Result};
use std::path::Path;

use crate::{ai_dir, config, providers, state};

pub fn run(providers_arg: Vec<String>, force: bool) -> Result<()> {
    let ai_dir_path = Path::new(".ai");
    config::ensure_default()?;
    if !ai_dir_path.exists() {
        anyhow::bail!(".ai directory does not exist. Run `dotai new` first");
    }
    if !ai_dir_path.join("AI.md").exists() {
        anyhow::bail!(".ai/AI.md not found");
    }

    let project = ai_dir::load(ai_dir_path)?;

    let config = match config::load()? {
        Some(cfg) => cfg,
        None => anyhow::bail!("Failed to load {}", config::path().display()),
    };

    let selected = if providers_arg.is_empty() {
        config.providers.clone()
    } else {
        providers_arg
    };

    let resolved = providers::resolve(&selected)?;

    let mut manifest = state::Manifest::load(ai_dir_path)?;
    let mut skipped = 0;

    if let Some(ai_md) = &project.ai_md {
        let mut content = ai_md.clone();
        if !content.ends_with('\n') {
            content.push('\n');
        }
        match manifest.write_if_changed("AGENTS.md", &content, force)? {
            state::Outcome::Written | state::Outcome::Adopted => println!("Wrote AGENTS.md"),
            state::Outcome::SkippedPreExisting => skipped += 1,
            state::Outcome::Unchanged => {}
        }
    }

    for provider in &resolved {
        let stats = provider
            .generate(&project, ai_dir_path, &config, &mut manifest, force)
            .with_context(|| format!("Failed to generate {} config", provider.name()))?;
        skipped += stats.skipped;
    }

    manifest.save(ai_dir_path)?;

    if skipped > 0 {
        println!(
            "Skipped {} pre-existing file(s) (use --force to overwrite)",
            skipped
        );
    }

    let names: Vec<String> = resolved.iter().map(|p| p.name().to_string()).collect();
    println!("Done. Generated: {}", names.join(", "));
    Ok(())
}
