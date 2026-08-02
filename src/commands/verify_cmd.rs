use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::frontmatter;
use crate::util;

pub fn run() -> Result<()> {
    let ai_dir = Path::new(".ai");
    if !ai_dir.exists() {
        anyhow::bail!(".ai directory does not exist. Run `dotai new` first");
    }

    let mut checked = 0usize;
    let mut by_kind: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

    scan_flat(
        &ai_dir.join("agents"),
        "agent",
        &mut checked,
        &mut by_kind,
        check_agent,
    )?;
    scan_flat(
        &ai_dir.join("commands"),
        "command",
        &mut checked,
        &mut by_kind,
        check_command,
    )?;
    scan_flat(
        &ai_dir.join("rules"),
        "rule",
        &mut checked,
        &mut by_kind,
        check_rule,
    )?;
    scan_skills(&ai_dir.join("skills"), "skill", &mut checked, &mut by_kind)?;

    by_kind.retain(|_, messages| !messages.is_empty());
    let total: usize = by_kind.values().map(Vec::len).sum();

    if total == 0 {
        println!("Verified {} files in .ai: all OK", checked);
        return Ok(());
    }
    for messages in by_kind.values() {
        for message in messages {
            eprintln!("{}", message);
        }
    }
    let noun = if total == 1 { "problem" } else { "problems" };
    eprintln!("{} {} found", total, noun);
    for kind in by_kind.keys() {
        eprintln!(
            "hint: run `dotai new {} help` for the available frontmatter parameters",
            kind
        );
    }
    std::process::exit(1);
}

fn scan_flat(
    dir: &Path,
    kind: &'static str,
    checked: &mut usize,
    by_kind: &mut BTreeMap<&'static str, Vec<String>>,
    check: impl Fn(&mut Vec<String>, &str, &str),
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in md_entries(dir)? {
        let path = entry.path();
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        *checked += 1;
        let messages = by_kind.entry(kind).or_default();
        check(messages, &name, &path.to_string_lossy());
    }
    Ok(())
}

fn md_entries(dir: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            !name.starts_with('.') && e.path().is_file() && name.ends_with(".md")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

fn scan_skills(
    dir: &Path,
    kind: &'static str,
    checked: &mut usize,
    by_kind: &mut BTreeMap<&'static str, Vec<String>>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                by_kind
                    .entry(kind)
                    .or_default()
                    .push(format!("{}: missing SKILL.md", path.to_string_lossy()));
                continue;
            }
            *checked += 1;
            let messages = by_kind.entry(kind).or_default();
            check_skill(messages, &name, &skill_md.to_string_lossy());
        } else if name.ends_with(".md") && name != "SKILL.md" {
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            *checked += 1;
            let messages = by_kind.entry(kind).or_default();
            check_skill(messages, &stem, &path.to_string_lossy());
        }
    }
    Ok(())
}

fn check_agent(errors: &mut Vec<String>, name: &str, path: &str) {
    let Some(parsed) = parse_file(errors, path) else {
        return;
    };
    check_common(errors, path, name, &parsed);
    check_unknown_keys(
        errors,
        path,
        &parsed.meta,
        &[
            "description",
            "model",
            "effort",
            "temperature",
            "mode",
            "allow",
            "deny",
        ],
    );
    if let Some(model) = frontmatter::get_str(&parsed.meta, "model") {
        if !matches!(model.as_str(), "small" | "medium" | "large")
            && model.chars().any(|c| c.is_whitespace())
        {
            errors.push(format!(
                "{}: model must be small|medium|large or a full model id",
                path
            ));
        }
    }
    if let Some(effort) = frontmatter::get_str(&parsed.meta, "effort") {
        if !matches!(effort.as_str(), "low" | "medium" | "high" | "xhigh" | "max") {
            errors.push(format!(
                "{}: effort must be one of low, medium, high, xhigh, max",
                path
            ));
        }
    }
    if let Some(temperature) = parsed
        .meta
        .get(serde_yaml::Value::String("temperature".to_string()))
    {
        if temperature.as_f64().is_none() {
            errors.push(format!("{}: temperature must be a number", path));
        }
    }
    if let Some(mode) = frontmatter::get_str(&parsed.meta, "mode") {
        if !matches!(mode.as_str(), "subagent" | "primary") {
            errors.push(format!("{}: mode must be subagent or primary", path));
        }
    }
}

fn check_skill(errors: &mut Vec<String>, name: &str, path: &str) {
    let Some(parsed) = parse_file(errors, path) else {
        return;
    };
    check_common(errors, path, name, &parsed);
    check_unknown_keys(
        errors,
        path,
        &parsed.meta,
        &["description", "allow", "paths"],
    );
}

fn check_command(errors: &mut Vec<String>, name: &str, path: &str) {
    let Some(parsed) = parse_file(errors, path) else {
        return;
    };
    check_common(errors, path, name, &parsed);
    check_unknown_keys(
        errors,
        path,
        &parsed.meta,
        &["description", "argument-hint", "agent"],
    );
}

fn check_rule(errors: &mut Vec<String>, name: &str, path: &str) {
    let Some(parsed) = parse_file(errors, path) else {
        return;
    };
    check_common(errors, path, name, &parsed);
    check_unknown_keys(
        errors,
        path,
        &parsed.meta,
        &["description", "globs", "always-apply"],
    );
    if let Some(always_apply) = parsed
        .meta
        .get(serde_yaml::Value::String("always-apply".to_string()))
    {
        if always_apply.as_bool().is_none() {
            errors.push(format!("{}: always-apply must be true or false", path));
        }
    }
}

fn check_common(errors: &mut Vec<String>, path: &str, name: &str, parsed: &frontmatter::Parsed) {
    if let Err(e) = util::validate_name(name) {
        errors.push(format!("{}: {}", path, e));
    }
    if frontmatter::get_str(&parsed.meta, "description").is_none() {
        errors.push(format!("{}: missing or empty `description`", path));
    }
    if parsed.body.trim().is_empty() {
        errors.push(format!("{}: empty body", path));
    }
}

fn check_unknown_keys(
    errors: &mut Vec<String>,
    path: &str,
    meta: &serde_yaml::Mapping,
    known: &[&str],
) {
    for key in meta.keys() {
        if let Some(name) = key.as_str() {
            if !known.contains(&name) {
                errors.push(format!("{}: unknown frontmatter key `{}`", path, name));
            }
        }
    }
}

fn parse_file(errors: &mut Vec<String>, path: &str) -> Option<frontmatter::Parsed> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            errors.push(format!("{}: {}", path, e));
            return None;
        }
    };
    match frontmatter::parse(&content) {
        Ok(parsed) => {
            if !content.trim_start().starts_with("---") {
                errors.push(format!(
                    "{}: missing frontmatter (file must start with ---)",
                    path
                ));
            }
            Some(parsed)
        }
        Err(e) => {
            errors.push(format!("{}: {}", path, e));
            None
        }
    }
}
