use anyhow::{Context, Result};
use std::path::Path;

use crate::model::Project;
use crate::providers::{self, Provider};
use crate::translate;

pub struct Cursor;

impl Provider for Cursor {
    fn name(&self) -> &str {
        "cursor"
    }

    fn generate(&self, project: &Project, _ai_dir: &Path) -> Result<()> {
        let out = std::path::PathBuf::from(".cursor");
        std::fs::create_dir_all(&out)
            .with_context(|| format!("Failed to create {}", out.display()))?;

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
                providers::write_md_with_fm(
                    &rules_dir.join(format!("{}.mdc", rule.name)),
                    &meta,
                    &rule.body,
                )?;
                n_rules += 1;
            }
        }
        removed += providers::remove_stale_files(&rules_dir, ".mdc", &rule_names)?;

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
                    meta.push(("model".to_string(), model.clone()));
                }
                if !agent.tools.is_empty()
                    && agent.tools.iter().all(|t| translate::is_readonly_tool(t))
                {
                    meta.push(("readonly".to_string(), "true".to_string()));
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
                if !skill.paths.is_empty() {
                    meta.push(("paths".to_string(), providers::yaml_list(&skill.paths)));
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
                let mut content = providers::GENERATED.to_string();
                content.push_str(&translate::translate_command_vars(
                    &cmd.body,
                    translate::VarStyle::Keep,
                ));
                content.push('\n');
                crate::util::write(&commands_dir.join(format!("{}.md", cmd.name)), &content)?;
                n_commands += 1;
            }
        }
        removed += providers::remove_stale_files(&commands_dir, ".md", &command_names)?;

        providers::summary(".cursor", n_rules, n_agents, n_skills, n_commands, removed);
        Ok(())
    }

    fn cleanup(&self) -> Result<usize> {
        let out = std::path::PathBuf::from(".cursor");
        let mut removed = 0;
        removed += providers::remove_stale_files(&out.join("rules"), ".mdc", &[])?;
        removed += providers::remove_stale_files(&out.join("agents"), ".md", &[])?;
        removed += providers::remove_stale_skills(&out.join("skills"), &[])?;
        removed += providers::remove_stale_files(&out.join("commands"), ".md", &[])?;
        providers::remove_empty(&out);
        Ok(removed)
    }
}
