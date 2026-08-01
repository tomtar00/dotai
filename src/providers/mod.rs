use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::model::{Project, Skill};
use crate::state::{is_legacy, Manifest, Outcome};

pub mod claude;
pub mod cursor;
pub mod opencode;

#[derive(Debug, Default)]
pub struct GenStats {
    pub written: usize,
    pub skipped: usize,
}

impl GenStats {
    pub fn count(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Written | Outcome::Adopted => self.written += 1,
            Outcome::SkippedPreExisting => self.skipped += 1,
            Outcome::Unchanged => {}
        }
    }
}

pub trait Provider {
    fn name(&self) -> &str;
    fn generate(
        &self,
        project: &Project,
        ai_dir: &Path,
        config: &Config,
        manifest: &mut Manifest,
        force: bool,
    ) -> Result<GenStats>;
    fn cleanup(&self, _manifest: &mut Manifest) -> Result<usize> {
        Ok(0)
    }
}

pub fn all() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(claude::Claude),
        Box::new(cursor::Cursor),
        Box::new(opencode::OpenCode),
    ]
}

pub fn known_names() -> Vec<String> {
    all().into_iter().map(|p| p.name().to_string()).collect()
}

pub fn resolve(names: &[String]) -> Result<Vec<Box<dyn Provider>>> {
    let known = known_names();
    for name in names {
        if !known.contains(name) {
            anyhow::bail!(
                "unknown provider '{}'. Known providers: {}",
                name,
                known.join(", ")
            );
        }
    }
    Ok(all()
        .into_iter()
        .filter(|p| names.contains(&p.name().to_string()))
        .collect())
}

/// Resolve an agent's `model:` frontmatter value. `small`/`medium`/`large` map
/// through the config's per-provider `models` table; anything else is passed
/// through verbatim. `None` means the model field should be omitted.
pub fn resolve_model(config: &Config, provider: &str, value: &str) -> Option<String> {
    match value {
        "small" | "medium" | "large" => config
            .models
            .get(provider)
            .and_then(|m| m.get(value))
            .cloned(),
        _ => Some(value.to_string()),
    }
}

pub fn frontmatter(entries: &[(&str, String)]) -> String {
    let mut s = String::from("---\n");
    for (k, v) in entries {
        if v.starts_with('\n') {
            s.push_str(&format!("{}:{}\n", k, v));
        } else {
            s.push_str(&format!("{}: {}\n", k, v));
        }
    }
    s.push_str("---\n");
    s
}

pub fn comma_list(items: &[String]) -> String {
    items.join(", ")
}

pub fn yaml_list(items: &[String]) -> String {
    let mut s = String::from("\n");
    for item in items {
        s.push_str(&format!("  - \"{}\"\n", item.replace('"', "\\\"")));
    }
    s
}

pub fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

pub fn write_with_manifest(
    manifest: &mut Manifest,
    path: &Path,
    content: &str,
    force: bool,
) -> Result<Outcome> {
    manifest.write_if_changed(&path.to_string_lossy(), content, force)
}

pub fn write_md_with_fm(
    manifest: &mut Manifest,
    path: &Path,
    meta: &[(&str, String)],
    body: &str,
    force: bool,
) -> Result<Outcome> {
    let mut content = frontmatter(meta);
    content.push_str(body);
    content.push('\n');
    write_with_manifest(manifest, path, &content, force)
}

/// Copy a skill's resources and write its SKILL.md. Returns how SKILL.md itself
/// was handled; resource copies are tracked in the manifest too.
pub fn write_skill(
    manifest: &mut Manifest,
    base: &Path,
    skill: &Skill,
    meta: &[(&str, String)],
    force: bool,
) -> Result<Outcome> {
    let dst = base.join(&skill.name);
    fs::create_dir_all(&dst).with_context(|| format!("Failed to create {}", dst.display()))?;
    if let Some(src) = &skill.src_dir {
        copy_resources(manifest, src, &dst, force)?;
    }
    let mut content = frontmatter(meta);
    content.push_str(&skill.body);
    content.push('\n');
    let outcome = write_with_manifest(manifest, &dst.join("SKILL.md"), &content, force)?;
    remove_extra_resources(manifest, skill.src_dir.as_deref(), &dst)?;
    Ok(outcome)
}

fn copy_resources(manifest: &mut Manifest, src: &Path, dst: &Path, force: bool) -> Result<()> {
    for entry in fs::read_dir(src).with_context(|| format!("Failed to read {}", src.display()))? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "SKILL.md" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&dst_path)
                .with_context(|| format!("Failed to create {}", dst_path.display()))?;
            copy_resources(manifest, &src_path, &dst_path, force)?;
        } else {
            let data = fs::read(&src_path)
                .with_context(|| format!("Failed to read {}", src_path.display()))?;
            manifest.write_if_changed_bytes(&dst_path.to_string_lossy(), &data, force)?;
        }
    }
    Ok(())
}

fn remove_extra_resources(manifest: &mut Manifest, src: Option<&Path>, dst: &Path) -> Result<()> {
    let entries = fs::read_dir(dst).with_context(|| format!("Failed to read {}", dst.display()))?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        if name == "SKILL.md" {
            continue;
        }
        let src_path = src.map(|s| s.join(&name));
        let rel = entry.path().to_string_lossy().to_string();
        if entry.file_type()?.is_dir() {
            if src_path.as_deref().map(|p| p.is_dir()).unwrap_or(false) {
                remove_extra_resources(manifest, src_path.as_deref(), &entry.path())?;
                remove_empty(&entry.path());
            } else if manifest.owns(&rel) || manifest.owns_any(&rel) || is_legacy(&entry.path()) {
                fs::remove_dir_all(entry.path())
                    .with_context(|| format!("Failed to remove {}", entry.path().display()))?;
                manifest.remove_prefix(&format!("{}/", rel));
            }
        } else if entry.file_type()?.is_file()
            && !src_path.as_deref().map(|p| p.is_file()).unwrap_or(false)
            && (manifest.owns(&rel) || is_legacy(&entry.path()))
        {
            fs::remove_file(entry.path())
                .with_context(|| format!("Failed to remove {}", entry.path().display()))?;
            manifest.mark_removed(&rel);
        }
    }
    Ok(())
}

pub fn remove_empty(dir: &Path) {
    if let Ok(mut entries) = fs::read_dir(dir) {
        if entries.next().is_none() {
            let _ = fs::remove_dir(dir);
        }
    }
}

pub fn remove_file_if_owned(manifest: &mut Manifest, path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let rel = path.to_string_lossy().to_string();
    if !(manifest.owns(&rel) || is_legacy(path)) {
        return Ok(0);
    }
    fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    manifest.mark_removed(&rel);
    Ok(1)
}

/// Remove files in `dir` (extension `ext`) whose stem is not in `keep`, but
/// only those dotai owns (tracked in the manifest or carrying the legacy
/// marker). User files are never touched.
pub fn remove_stale_files(
    manifest: &mut Manifest,
    dir: &Path,
    ext: &str,
    keep: &[String],
) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut removed = 0;
    let entries = fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !name.ends_with(ext) {
            continue;
        }
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let stem = Path::new(&name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if keep.contains(&stem) {
            continue;
        }
        let path = entry.path();
        let rel = path.to_string_lossy().to_string();
        if !(manifest.owns(&rel) || is_legacy(&path)) {
            continue;
        }
        fs::remove_file(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
        manifest.mark_removed(&rel);
        removed += 1;
    }
    remove_empty(dir);
    Ok(removed)
}

/// Remove generated skill directories (manifest-owned or legacy SKILL.md) not
/// in `keep`.
pub fn remove_stale_skills(manifest: &mut Manifest, base: &Path, keep: &[String]) -> Result<usize> {
    if !base.exists() {
        return Ok(0);
    }
    let mut removed = 0;
    let entries =
        fs::read_dir(base).with_context(|| format!("Failed to read {}", base.display()))?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let skill_md = entry.path().join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let rel = skill_md.to_string_lossy().to_string();
        if keep.contains(&name) || !(manifest.owns(&rel) || is_legacy(&skill_md)) {
            continue;
        }
        fs::remove_dir_all(entry.path())
            .with_context(|| format!("Failed to remove {}", entry.path().display()))?;
        manifest.remove_prefix(&format!("{}/", entry.path().to_string_lossy()));
        removed += 1;
    }
    remove_empty(base);
    Ok(removed)
}

pub fn summary(
    name: &str,
    rules: usize,
    agents: usize,
    skills: usize,
    commands: usize,
    removed: usize,
) {
    let mut parts = Vec::new();
    if rules > 0 {
        parts.push(format!(
            "{} rule{}",
            rules,
            if rules == 1 { "" } else { "s" }
        ));
    }
    if agents > 0 {
        parts.push(format!(
            "{} agent{}",
            agents,
            if agents == 1 { "" } else { "s" }
        ));
    }
    if skills > 0 {
        parts.push(format!(
            "{} skill{}",
            skills,
            if skills == 1 { "" } else { "s" }
        ));
    }
    if commands > 0 {
        parts.push(format!(
            "{} command{}",
            commands,
            if commands == 1 { "" } else { "s" }
        ));
    }
    let mut msg = format!(
        "  Synced {} ({})",
        name,
        if parts.is_empty() {
            "no items".to_string()
        } else {
            parts.join(", ")
        }
    );
    if removed > 0 {
        msg.push_str(&format!(
            "; removed {} stale file{}",
            removed,
            if removed == 1 { "" } else { "s" }
        ));
    }
    println!("{}", msg);
}
