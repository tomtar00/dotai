use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::model::Project;
use crate::providers::{self, Provider};
use crate::translate::{self, tool_to_claude};

pub struct Claude;

impl Provider for Claude {
    fn name(&self) -> &str {
        "claude"
    }

    fn generate(&self, project: &Project, _ai_dir: &Path) -> Result<()> {
        let out = std::path::PathBuf::from(".claude");
        std::fs::create_dir_all(&out)
            .with_context(|| format!("Failed to create {}", out.display()))?;

        if project.ai_md.is_some() {
            let mut content = providers::GENERATED.to_string();
            content.push_str("\n@../AGENTS.md\n");
            crate::util::write(&out.join("CLAUDE.md"), &content)?;
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
                providers::write_md_with_fm(
                    &rules_dir.join(format!("{}.md", rule.name)),
                    &meta,
                    &rule.body,
                )?;
                n_rules += 1;
            }
        }
        removed += providers::remove_stale_files(&rules_dir, ".md", &rule_names)?;

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
                if !agent.tools.is_empty() {
                    let tools: Vec<String> = providers::dedupe(
                        agent
                            .tools
                            .iter()
                            .filter_map(|t| tool_to_claude(t))
                            .collect(),
                    );
                    if !tools.is_empty() {
                        meta.push(("tools".to_string(), providers::comma_list(&tools)));
                    }
                }
                if let Some(model) = &agent.model {
                    meta.push(("model".to_string(), translate::strip_provider_prefix(model)));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                providers::write_md_with_fm(
                    &agents_dir.join(format!("{}.md", agent.name)),
                    &meta,
                    &agent.body,
                )?;
                n_agents += 1;
            }
        }
        removed += providers::remove_stale_files(&agents_dir, ".md", &agent_names)?;

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
                if !skill.allowed_tools.is_empty() {
                    let tools: Vec<String> = providers::dedupe(
                        skill
                            .allowed_tools
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
                providers::write_skill(&skills_dir, skill, &meta)?;
                n_skills += 1;
            }
        }
        removed += providers::remove_stale_skills(&skills_dir, &skill_names)?;

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
                providers::write_md_with_fm(
                    &commands_dir.join(format!("{}.md", cmd.name)),
                    &meta,
                    &body,
                )?;
                n_commands += 1;
            }
        }
        removed += providers::remove_stale_files(&commands_dir, ".md", &command_names)?;

        providers::summary(".claude", n_rules, n_agents, n_skills, n_commands, removed);
        Ok(())
    }

    fn cleanup(&self) -> Result<usize> {
        let out = std::path::PathBuf::from(".claude");
        let mut removed = 0;
        let claude_md = out.join("CLAUDE.md");
        if claude_md.exists() {
            let content = fs::read_to_string(&claude_md).unwrap_or_default();
            if providers::is_generated(&content) {
                fs::remove_file(&claude_md)
                    .with_context(|| format!("Failed to remove {}", claude_md.display()))?;
                removed += 1;
            }
        }
        removed += providers::remove_stale_files(&out.join("rules"), ".md", &[])?;
        removed += providers::remove_stale_files(&out.join("agents"), ".md", &[])?;
        removed += providers::remove_stale_skills(&out.join("skills"), &[])?;
        removed += providers::remove_stale_files(&out.join("commands"), ".md", &[])?;
        providers::remove_empty(&out);
        Ok(removed)
    }
}
