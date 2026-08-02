use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::model::Project;
use crate::providers::{self, GenStats, Provider};
use crate::state::Manifest;
use crate::translate;

pub struct Cursor;

impl Provider for Cursor {
    fn name(&self) -> &str {
        "cursor"
    }

    fn generate(
        &self,
        project: &Project,
        _ai_dir: &Path,
        config: &Config,
        manifest: &mut Manifest,
        force: bool,
    ) -> Result<GenStats> {
        let out = std::path::PathBuf::from(".cursor");
        std::fs::create_dir_all(&out)
            .with_context(|| format!("Failed to create {}", out.display()))?;

        let mut stats = GenStats::default();
        let mut removed = 0;

        let mut n_rules = 0;
        let rule_names: Vec<String> = project.rules.iter().map(|r| r.name.clone()).collect();
        let rules_dir = out.join("rules");
        if !project.rules.is_empty() {
            std::fs::create_dir_all(&rules_dir)
                .with_context(|| format!("Failed to create {}", rules_dir.display()))?;
            for rule in &project.rules {
                let mut meta = Vec::new();
                if let Some(description) = &rule.description {
                    meta.push(("description".to_string(), description.clone()));
                }
                if !rule.globs.is_empty() {
                    meta.push(("globs".to_string(), providers::yaml_list(&rule.globs)));
                }
                if rule.always_apply {
                    meta.push(("alwaysApply".to_string(), "true".to_string()));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                let outcome = providers::write_md_with_fm(
                    manifest,
                    &rules_dir.join(format!("{}.mdc", rule.name)),
                    &meta,
                    &rule.body,
                    force,
                )?;
                stats.count(outcome);
                n_rules += 1;
            }
        }
        removed += providers::remove_stale_files(manifest, &rules_dir, ".mdc", &rule_names)?;

        let mut n_agents = 0;
        let agent_names: Vec<String> = project.agents.iter().map(|a| a.name.clone()).collect();
        let agents_dir = out.join("agents");
        if !project.agents.is_empty() {
            std::fs::create_dir_all(&agents_dir)
                .with_context(|| format!("Failed to create {}", agents_dir.display()))?;
            for agent in &project.agents {
                let mut meta = vec![
                    ("name".to_string(), agent.name.clone()),
                    ("description".to_string(), agent.description.clone()),
                ];
                if let Some(model) = &agent.model {
                    if let Some(resolved) = providers::resolve_model(config, "cursor", model) {
                        let model_str = match &agent.effort {
                            Some(effort) if !resolved.contains('[') => {
                                format!("{}[effort={}]", resolved, effort)
                            }
                            _ => resolved,
                        };
                        meta.push(("model".to_string(), model_str));
                    }
                }
                if agent.deny.iter().any(|t| t == "edit") {
                    meta.push(("readonly".to_string(), "true".to_string()));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                let outcome = providers::write_md_with_fm(
                    manifest,
                    &agents_dir.join(format!("{}.md", agent.name)),
                    &meta,
                    &agent.body,
                    force,
                )?;
                stats.count(outcome);
                n_agents += 1;
            }
        }
        removed += providers::remove_stale_files(manifest, &agents_dir, ".md", &agent_names)?;

        let mut n_skills = 0;
        let skill_names: Vec<String> = project.skills.iter().map(|s| s.name.clone()).collect();
        let skills_dir = out.join("skills");
        if !project.skills.is_empty() {
            std::fs::create_dir_all(&skills_dir)
                .with_context(|| format!("Failed to create {}", skills_dir.display()))?;
            for skill in &project.skills {
                let mut meta = vec![
                    ("name".to_string(), skill.name.clone()),
                    ("description".to_string(), skill.description.clone()),
                ];
                if !skill.paths.is_empty() {
                    meta.push(("paths".to_string(), providers::yaml_list(&skill.paths)));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                let outcome = providers::write_skill(manifest, &skills_dir, skill, &meta, force)?;
                stats.count(outcome);
                n_skills += 1;
            }
        }
        removed += providers::remove_stale_skills(manifest, &skills_dir, &skill_names)?;

        let mut n_commands = 0;
        let command_names: Vec<String> = project.commands.iter().map(|c| c.name.clone()).collect();
        let commands_dir = out.join("commands");
        if !project.commands.is_empty() {
            std::fs::create_dir_all(&commands_dir)
                .with_context(|| format!("Failed to create {}", commands_dir.display()))?;
            for cmd in &project.commands {
                let content =
                    translate::translate_command_vars(&cmd.body, translate::VarStyle::Keep);
                let outcome = providers::write_with_manifest(
                    manifest,
                    &commands_dir.join(format!("{}.md", cmd.name)),
                    &format!("{}\n", content),
                    force,
                )?;
                stats.count(outcome);
                n_commands += 1;
            }
        }
        removed += providers::remove_stale_files(manifest, &commands_dir, ".md", &command_names)?;

        providers::summary(".cursor", n_rules, n_agents, n_skills, n_commands, removed);
        Ok(stats)
    }

    fn cleanup(&self, manifest: &mut Manifest) -> Result<usize> {
        let out = std::path::PathBuf::from(".cursor");
        let mut removed = 0;
        removed += providers::remove_stale_files(manifest, &out.join("rules"), ".mdc", &[])?;
        removed += providers::remove_stale_files(manifest, &out.join("agents"), ".md", &[])?;
        removed += providers::remove_stale_skills(manifest, &out.join("skills"), &[])?;
        removed += providers::remove_stale_files(manifest, &out.join("commands"), ".md", &[])?;
        providers::remove_empty(&out);
        Ok(removed)
    }
}
