use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::model::Project;
use crate::providers::{self, GenStats, Provider};
use crate::state::Manifest;
use crate::translate::{self, tool_to_claude};

pub struct Claude;

impl Provider for Claude {
    fn name(&self) -> &str {
        "claude"
    }

    fn generate(
        &self,
        project: &Project,
        _ai_dir: &Path,
        config: &Config,
        manifest: &mut Manifest,
        force: bool,
    ) -> Result<GenStats> {
        let out = std::path::PathBuf::from(".claude");
        std::fs::create_dir_all(&out)
            .with_context(|| format!("Failed to create {}", out.display()))?;

        let mut stats = GenStats::default();

        if project.ai_md.is_some() {
            let outcome = providers::write_with_manifest(
                manifest,
                &out.join("CLAUDE.md"),
                "@../AGENTS.md\n",
                force,
            )?;
            stats.count(outcome);
        }

        let mut removed = 0;

        let mut n_rules = 0;
        let rule_names: Vec<String> = project.rules.iter().map(|r| r.name.clone()).collect();
        let rules_dir = out.join("rules");
        if !project.rules.is_empty() {
            std::fs::create_dir_all(&rules_dir)
                .with_context(|| format!("Failed to create {}", rules_dir.display()))?;
            for rule in &project.rules {
                let mut meta = Vec::new();
                if !rule.globs.is_empty() {
                    meta.push(("globs", format!("\"{}\"", rule.globs.join(", "))));
                }
                let outcome = providers::write_md_with_fm(
                    manifest,
                    &rules_dir.join(format!("{}.md", rule.name)),
                    &meta,
                    &rule.body,
                    force,
                )?;
                stats.count(outcome);
                n_rules += 1;
            }
        }
        removed += providers::remove_stale_files(manifest, &rules_dir, ".md", &rule_names)?;

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
                if !agent.allow.is_empty() {
                    let tools: Vec<String> = providers::dedupe(
                        agent
                            .allow
                            .iter()
                            .filter_map(|t| tool_to_claude(t))
                            .collect(),
                    );
                    if !tools.is_empty() {
                        meta.push(("tools".to_string(), providers::comma_list(&tools)));
                    }
                }
                if !agent.deny.is_empty() {
                    let tools: Vec<String> = providers::dedupe(
                        agent
                            .deny
                            .iter()
                            .filter_map(|t| tool_to_claude(t))
                            .collect(),
                    );
                    if !tools.is_empty() {
                        meta.push((
                            "disallowed-tools".to_string(),
                            providers::comma_list(&tools),
                        ));
                    }
                }
                if let Some(model) = &agent.model {
                    if let Some(resolved) = providers::resolve_model(config, "claude", model) {
                        meta.push((
                            "model".to_string(),
                            translate::strip_provider_prefix(&resolved),
                        ));
                    }
                }
                if let Some(effort) = &agent.effort {
                    meta.push(("effort".to_string(), effort.clone()));
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
                if !skill.allow.is_empty() {
                    let tools: Vec<String> = providers::dedupe(
                        skill
                            .allow
                            .iter()
                            .filter_map(|t| tool_to_claude(t))
                            .collect(),
                    );
                    if !tools.is_empty() {
                        meta.push(("allowed-tools".to_string(), providers::comma_list(&tools)));
                    }
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
                let mut meta = Vec::new();
                if !cmd.description.is_empty() {
                    meta.push(("description".to_string(), cmd.description.clone()));
                }
                if let Some(hint) = &cmd.argument_hint {
                    meta.push(("argument-hint".to_string(), hint.clone()));
                }
                let named = translate::command_named_args(&cmd.body);
                if !named.is_empty() {
                    meta.push(("arguments".to_string(), format!("[{}]", named.join(", "))));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                let body =
                    translate::translate_command_vars(&cmd.body, translate::VarStyle::Dollar);
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

        providers::summary(".claude", n_rules, n_agents, n_skills, n_commands, removed);
        Ok(stats)
    }

    fn cleanup(&self, manifest: &mut Manifest) -> Result<usize> {
        let out = std::path::PathBuf::from(".claude");
        let mut removed = 0;
        removed += providers::remove_file_if_owned(manifest, &out.join("CLAUDE.md"))?;
        removed += providers::remove_stale_files(manifest, &out.join("rules"), ".md", &[])?;
        removed += providers::remove_stale_files(manifest, &out.join("agents"), ".md", &[])?;
        removed += providers::remove_stale_skills(manifest, &out.join("skills"), &[])?;
        removed += providers::remove_stale_files(manifest, &out.join("commands"), ".md", &[])?;
        providers::remove_empty(&out);
        Ok(removed)
    }
}
