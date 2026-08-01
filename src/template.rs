use anyhow::{Context, Result};
use std::fs;

use crate::util;

pub const DEFAULT_AI_MD: &str = r#"# AI Configuration

This file is the single source of truth for AI agent behavior in this project.
It is copied verbatim to AGENTS.md (and imported by .claude/CLAUDE.md) by `dotai gen`.

## About

<!-- Describe your project, tech stack, and architecture here -->

## Guidelines

- Follow existing code conventions and patterns
- Write unit tests for all new functionality
- Keep functions small and focused on a single responsibility

## Communication

- Be concise and direct
- Explain reasoning behind suggestions and changes
- Ask for clarification when requirements are ambiguous

## Project rules

Scoped rules live in `rules/*.md` and are generated into provider-native rule
formats (`.claude/rules/`, `.cursor/rules/`). Create them with
`dotai new rule <name>`.
"#;

pub const AGENT_SCAFFOLD: &str = r#"---
description: One sentence on when to delegate to this agent.
tools: read, grep, glob, list
---

You are {name}. Describe your role, behavior, and output format here.

- What you are responsible for
- How you approach tasks
- What you return
"#;

pub const SKILL_SCAFFOLD: &str = r#"---
description: What this skill does and when to use it. Use when the user asks for X, mentions Y, or works on files matching Z.
---

# {name}

## Instructions

- Step-by-step guidance

## Examples

- Concrete examples of inputs and outputs
"#;

pub const COMMAND_SCAFFOLD: &str = r#"---
description: One sentence describing what this command does.
argument-hint: "[your input]"
---

Write the prompt body here. Use {{input}} for everything the user typed after
the command, or {{1}} / {{2}} for positional arguments.
"#;

pub const RULE_SCAFFOLD: &str = r#"---
description: What this rule covers.
globs: "**/*.{ts,tsx}"
always-apply: false
---

Write the rule content here. It applies to the files matched by `globs`
(a comma-separated list of glob patterns, or omit `globs` to apply everywhere).
"#;

pub fn ensure_template() -> Result<std::path::PathBuf> {
    let t = util::template_dir();
    if !t.exists() {
        fs::create_dir_all(&t).with_context(|| format!("Failed to create {}", t.display()))?;
        for sub in ["rules", "agents", "skills", "commands"] {
            fs::create_dir_all(t.join(sub))
                .with_context(|| format!("Failed to create {}", t.join(sub).display()))?;
        }
        util::write(&t.join("AI.md"), DEFAULT_AI_MD)?;
        println!("Created blank template at {}", t.display());
    }
    Ok(t)
}

pub fn scaffold(kind: &str, name: &str) -> Result<String> {
    let override_path = util::template_dir().join(kind).join(".scaffold.md");
    let content = if override_path.exists() {
        fs::read_to_string(&override_path)
            .with_context(|| format!("Failed to read {}", override_path.display()))?
    } else {
        match kind {
            "agents" => AGENT_SCAFFOLD.to_string(),
            "skills" => SKILL_SCAFFOLD.to_string(),
            "commands" => COMMAND_SCAFFOLD.to_string(),
            "rules" => RULE_SCAFFOLD.to_string(),
            _ => anyhow::bail!("unknown scaffold kind: {}", kind),
        }
    };
    Ok(content.replace("{name}", name))
}
