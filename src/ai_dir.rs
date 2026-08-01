use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::frontmatter;
use crate::model::{Agent, Command, Project, Rule, Skill};

pub fn load(ai_dir: &Path) -> Result<Project> {
    let mut project = Project {
        ai_md: None,
        rules: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        commands: Vec::new(),
    };

    let ai_md = ai_dir.join("AI.md");
    if ai_md.exists() {
        project.ai_md = Some(
            fs::read_to_string(&ai_md)
                .with_context(|| format!("Failed to read {}", ai_md.display()))?
                .trim()
                .to_string(),
        );
    }

    project.rules = load_dir_files(&ai_dir.join("rules"), |name, meta, body| Rule {
        name,
        description: frontmatter::get_str(&meta, "description"),
        globs: frontmatter::get_list(&meta, "globs"),
        always_apply: frontmatter::get_bool(&meta, "always-apply"),
        body,
    })?;

    project.agents = load_dir_files(&ai_dir.join("agents"), |name, meta, body| Agent {
        name,
        description: frontmatter::get_str(&meta, "description").unwrap_or_default(),
        model: frontmatter::get_str(&meta, "model"),
        temperature: meta
            .get(serde_yaml::Value::String("temperature".to_string()))
            .and_then(|v| v.as_f64()),
        mode: frontmatter::get_str(&meta, "mode"),
        tools: frontmatter::get_list(&meta, "tools"),
        body,
    })?;

    project.skills = load_skills(&ai_dir.join("skills"))?;

    project.commands = load_dir_files(&ai_dir.join("commands"), |name, meta, body| Command {
        name,
        description: frontmatter::get_str(&meta, "description").unwrap_or_default(),
        argument_hint: frontmatter::get_str(&meta, "argument-hint"),
        agent: frontmatter::get_str(&meta, "agent"),
        body,
    })?;

    Ok(project)
}

fn load_dir_files<T>(
    dir: &Path,
    build: impl Fn(String, serde_yaml::Mapping, String) -> T,
) -> Result<Vec<T>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            e.path().is_file() && name.ends_with(".md") && !name.starts_with('.')
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut items = Vec::new();
    for entry in entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let parsed = frontmatter::parse(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        if parsed.meta.is_empty() && parsed.body.is_empty() {
            continue;
        }
        items.push(build(name, parsed.meta, parsed.body));
    }
    Ok(items)
}

fn load_skills(dir: &Path) -> Result<Vec<Skill>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read {}", dir.display()))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut skills = Vec::new();
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = fs::read_to_string(&skill_md)
                .with_context(|| format!("Failed to read {}", skill_md.display()))?;
            let parsed = frontmatter::parse(&content)
                .with_context(|| format!("Failed to parse {}", skill_md.display()))?;
            skills.push(Skill {
                name: name.clone(),
                description: frontmatter::get_str(&parsed.meta, "description").unwrap_or_default(),
                allowed_tools: frontmatter::get_list(&parsed.meta, "allowed-tools"),
                paths: frontmatter::get_list(&parsed.meta, "paths"),
                body: parsed.body,
                src_dir: Some(path),
            });
        } else if name.ends_with(".md") && name != "SKILL.md" {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let parsed = frontmatter::parse(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))?;
            skills.push(Skill {
                name: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                description: frontmatter::get_str(&parsed.meta, "description").unwrap_or_default(),
                allowed_tools: frontmatter::get_list(&parsed.meta, "allowed-tools"),
                paths: frontmatter::get_list(&parsed.meta, "paths"),
                body: parsed.body,
                src_dir: None,
            });
        }
    }
    Ok(skills)
}
