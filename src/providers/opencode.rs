use anyhow::{Context, Result};
use std::path::Path;

use crate::model::Project;
use crate::providers::{self, Provider};
use crate::translate;

pub struct OpenCode;

impl Provider for OpenCode {
    fn name(&self) -> &str {
        "opencode"
    }

    fn generate(&self, project: &Project, _ai_dir: &Path) -> Result<()> {
        let out = std::path::PathBuf::from(".opencode");
        std::fs::create_dir_all(&out)
            .with_context(|| format!("Failed to create {}", out.display()))?;

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
                    if model != "inherit" {
                        meta.push(("model".to_string(), model.clone()));
                    }
                }
                if let Some(temp) = &agent.temperature {
                    meta.push(("temperature".to_string(), temp.to_string()));
                }
                if !agent.tools.is_empty()
                    && agent.tools.iter().all(|t| translate::is_readonly_tool(t))
                {
                    meta.push((
                        "permission".to_string(),
                        "\n  edit: deny\n  bash: deny".to_string(),
                    ));
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
                let meta: Vec<(&str, String)> = vec![
                    ("name", skill.name.clone()),
                    ("description", skill.description.clone()),
                ];
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
                if let Some(agent) = &cmd.agent {
                    meta.push(("agent".to_string(), agent.clone()));
                }
                let meta: Vec<(&str, String)> =
                    meta.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                let body =
                    translate::translate_command_vars(&cmd.body, translate::VarStyle::OpenCode);
                providers::write_md_with_fm(
                    &commands_dir.join(format!("{}.md", cmd.name)),
                    &meta,
                    &body,
                )?;
                n_commands += 1;
            }
        }
        removed += providers::remove_stale_files(&commands_dir, ".md", &command_names)?;

        if !project.rules.is_empty() {
            println!("  .opencode: rules skipped (opencode has no rules directory; use AI.md)");
        }

        providers::summary(".opencode", 0, n_agents, n_skills, n_commands, removed);
        Ok(())
    }

    fn cleanup(&self) -> Result<usize> {
        let out = std::path::PathBuf::from(".opencode");
        let mut removed = 0;
        removed += providers::remove_stale_files(&out.join("agents"), ".md", &[])?;
        removed += providers::remove_stale_skills(&out.join("skills"), &[])?;
        removed += providers::remove_stale_files(&out.join("commands"), ".md", &[])?;
        providers::remove_empty(&out);
        Ok(removed)
    }
}
