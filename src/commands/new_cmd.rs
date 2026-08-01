use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::{config, template, util};

pub enum NewKind {
    Agent(String),
    Skill(String),
    Command(String),
    Rule(String),
    Ai,
}

pub fn run(kind: Option<NewKind>) -> Result<()> {
    match kind {
        None => bootstrap(),
        Some(NewKind::Agent(name)) => create_item("agents", "agent", &name),
        Some(NewKind::Skill(name)) => create_item("skills", "skill", &name),
        Some(NewKind::Command(name)) => create_item("commands", "command", &name),
        Some(NewKind::Rule(name)) => create_item("rules", "rule", &name),
        Some(NewKind::Ai) => create_ai_md(),
    }
}

pub fn bootstrap() -> Result<()> {
    let ai_dir = Path::new(".ai");
    if ai_dir.exists() {
        anyhow::bail!(".ai directory already exists");
    }

    let tmpl = template::ensure_template()?;
    util::copy_dir(&tmpl, ai_dir, true)
        .with_context(|| format!("Failed to copy template from {}", tmpl.display()))?;
    println!("Created .ai from template at {}", tmpl.display());

    if config::ensure_default()? {
        println!(
            "Created {} with default providers",
            config::path().display()
        );
    }

    println!(
        "Next: add items with `dotai new agent|skill|command|rule <name>`,\nthen generate provider configs with `dotai gen`."
    );
    Ok(())
}

fn ensure_ai() -> Result<()> {
    if !Path::new(".ai").exists() {
        bootstrap()?;
    }
    Ok(())
}

fn create_item(dir: &str, kind: &str, name: &str) -> Result<()> {
    ensure_ai()?;
    validate_name(name)?;
    let path = PathBuf::from(".ai").join(dir).join(format!("{}.md", name));
    if path.exists() {
        anyhow::bail!("{} '{}' already exists at {}", kind, name, path.display());
    }
    util::write(&path, &template::scaffold(dir, name)?)?;
    println!("Created {} '{}' at {}", kind, name, path.display());
    Ok(())
}

fn create_ai_md() -> Result<()> {
    ensure_ai()?;
    let path = PathBuf::from(".ai").join("AI.md");
    if path.exists() {
        anyhow::bail!("{} already exists", path.display());
    }
    util::write(&path, template::DEFAULT_AI_MD)?;
    println!("Created {}", path.display());
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    const RESERVED: &[&str] = &[
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !RESERVED.contains(&name);
    if !ok {
        anyhow::bail!(
            "invalid name '{}': use lowercase letters, digits and hyphens (e.g. code-review)",
            name
        );
    }
    Ok(())
}
