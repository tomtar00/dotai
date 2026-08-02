use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::model::Project;
use crate::providers::{self, GenStats, Provider};
use crate::state::Manifest;
use crate::translate;

pub struct OpenCode;

impl Provider for OpenCode {
    fn name(&self) -> &str {
        "opencode"
    }

    fn generate(
        &self,
        project: &Project,
        _ai_dir: &Path,
        config: &Config,
        manifest: &mut Manifest,
        force: bool,
    ) -> Result<GenStats> {
        let out = std::path::PathBuf::from(".opencode");
        std::fs::create_dir_all(&out)
            .with_context(|| format!("Failed to create {}", out.display()))?;

        let mut stats = GenStats::default();
        let mut removed = 0;

        let mut n_agents = 0;
        let agent_names: Vec<String> = project.agents.iter().map(|a| a.name.clone()).collect();
        let agents_dir = out.join("agents");
        if !project.agents.is_empty() {
            std::fs::create_dir_all(&agents_dir)
                .with_context(|| format!("Failed to create {}", agents_dir.display()))?;
            for agent in &project.agents {
                let mut meta = Vec::new();
                if !agent.description.is_empty() {
                    meta.push(("description".to_string(), agent.description.clone()));
                }
                meta.push((
                    "mode".to_string(),
                    agent.mode.clone().unwrap_or_else(|| "subagent".to_string()),
                ));
                if let Some(model) = &agent.model {
                    if let Some(resolved) = providers::resolve_model(config, "opencode", model) {
                        if resolved != "inherit" {
                            meta.push(("model".to_string(), resolved));
                        }
                    }
                }
                if let Some(effort) = &agent.effort {
                    meta.push(("reasoningEffort".to_string(), effort.clone()));
                }
                if let Some(temp) = &agent.temperature {
                    meta.push(("temperature".to_string(), temp.to_string()));
                }
                if !agent.allow.is_empty() || !agent.deny.is_empty() {
                    let mut rules = Vec::new();
                    for tool in providers::dedupe(agent.allow.clone()) {
                        rules.push(format!("  {}: allow", tool));
                    }
                    for tool in providers::dedupe(agent.deny.clone()) {
                        rules.push(format!("  {}: deny", tool));
                    }
                    if !rules.is_empty() {
                        meta.push(("permission".to_string(), format!("\n{}", rules.join("\n"))));
                    }
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
                let meta: Vec<(&str, String)> = vec![
                    ("name", skill.name.clone()),
                    ("description", skill.description.clone()),
                ];
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
                let mut meta = Vec::new();
                if !cmd.description.is_empty() {
                    meta.push(("description".to_string(), cmd.description.clone()));
                }
                if let Some(agent) = &cmd.agent {
                    meta.push(("agent".to_string(), agent.clone()));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                let body =
                    translate::translate_command_vars(&cmd.body, translate::VarStyle::OpenCode);
                let outcome = providers::write_md_with_fm(
                    manifest,
                    &commands_dir.join(format!("{}.md", cmd.name)),
                    &meta,
                    &body,
                    force,
                )?;
                stats.count(outcome);
                n_commands += 1;
            }
        }
        removed += providers::remove_stale_files(manifest, &commands_dir, ".md", &command_names)?;

        if !project.rules.is_empty() {
            println!("  .opencode: rules skipped (opencode has no rules directory; use AI.md)");
        }

        providers::summary(".opencode", 0, n_agents, n_skills, n_commands, removed);
        Ok(stats)
    }

    fn cleanup(&self, manifest: &mut Manifest) -> Result<usize> {
        let out = std::path::PathBuf::from(".opencode");
        let mut removed = 0;
        removed += providers::remove_stale_files(manifest, &out.join("agents"), ".md", &[])?;
        removed += providers::remove_stale_skills(manifest, &out.join("skills"), &[])?;
        removed += providers::remove_stale_files(manifest, &out.join("commands"), ".md", &[])?;
        providers::remove_empty(&out);
        Ok(removed)
    }
}
