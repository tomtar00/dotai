use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::{config, frontmatter, help, template, util};

pub enum NewKind {
    Agent(Option<String>, Option<String>, Option<String>),
    Skill(Option<String>),
    Command(Option<String>),
    Rule(Option<String>),
    Ai,
    Help(Option<String>),
}

pub fn run(kind: Option<NewKind>) -> Result<()> {
    match kind {
        None => bootstrap(),
        Some(NewKind::Help(item)) => {
            help::item(item.as_deref().unwrap_or(""));
            Ok(())
        }
        Some(NewKind::Agent(name, _, _)) if name.as_deref() == Some("help") => {
            help::agent();
            Ok(())
        }
        Some(NewKind::Skill(name)) if name.as_deref() == Some("help") => {
            help::skill();
            Ok(())
        }
        Some(NewKind::Command(name)) if name.as_deref() == Some("help") => {
            help::command();
            Ok(())
        }
        Some(NewKind::Rule(name)) if name.as_deref() == Some("help") => {
            help::rule();
            Ok(())
        }
        Some(kind) => {
            config::ensure_default()?;
            match kind {
                NewKind::Agent(name, model, effort) => wizard_agent(name, model, effort),
                NewKind::Skill(name) => wizard_skill(name),
                NewKind::Command(name) => wizard_command(name),
                NewKind::Rule(name) => wizard_rule(name),
                NewKind::Ai => create_ai_md(),
                NewKind::Help(_) => unreachable!("help handled above"),
            }
        }
    }
}

pub fn bootstrap() -> Result<()> {
    let ai_dir = Path::new(".ai");
    if ai_dir.exists() {
        anyhow::bail!(".ai directory already exists");
    }

    let tmpl = template::ensure_template()?;
    util::copy_dir(&tmpl, ai_dir, true)
        .with_context(|| format!("Failed to copy template from {}", tmpl.display()))?;
    println!("Created .ai from template at {}", tmpl.display());

    if config::ensure_default()? {
        println!(
            "Created {} with default providers and models",
            config::path().display()
        );
    }

    println!(
        "Next: add items with `dotai new agent|skill|command|rule <name>`,\nthen generate provider configs with `dotai gen`."
    );
    Ok(())
}

fn ensure_ai() -> Result<()> {
    if !Path::new(".ai").exists() {
        bootstrap()?;
    }
    Ok(())
}

fn wizard_agent(name: Option<String>, model: Option<String>, effort: Option<String>) -> Result<()> {
    let mut w = Wizard::new();
    let name = match name {
        Some(name) => {
            validate_name(&name)?;
            name
        }
        None => w.ask_name()?,
    };
    let description = w
        .ask(
            "description",
            Some("One sentence on when to delegate to this agent."),
        )
        .unwrap_or_default();
    let model = match model {
        Some(model) => model,
        None => w
            .ask("model (small|medium|large)", Some("medium"))
            .unwrap_or_else(|| "medium".to_string()),
    };
    let effort = match effort {
        Some(effort) => effort,
        None => w
            .ask("effort (low|medium|high|xhigh|max)", Some("medium"))
            .unwrap_or_else(|| "medium".to_string()),
    };
    let temperature = w.ask_f64("temperature (optional)")?;
    let mode = w
        .ask("mode (subagent|primary)", Some("subagent"))
        .unwrap_or_else(|| "subagent".to_string());
    let allow = w
        .ask(
            "allow (comma-separated tools)",
            Some("read, grep, glob, list"),
        )
        .unwrap_or_else(|| "read, grep, glob, list".to_string());
    let deny = w
        .ask("deny (comma-separated tools, optional)", None)
        .unwrap_or_default();

    let pairs = vec![
        ("description".to_string(), description),
        ("model".to_string(), model),
        ("effort".to_string(), effort),
        (
            "temperature".to_string(),
            temperature.map(|t| t.to_string()).unwrap_or_default(),
        ),
        ("mode".to_string(), mode),
        ("allow".to_string(), allow),
        ("deny".to_string(), deny),
    ];
    write_item("agents", "agent", &name, &pairs)
}

fn wizard_skill(name: Option<String>) -> Result<()> {
    let mut w = Wizard::new();
    let name = match name {
        Some(name) => {
            validate_name(&name)?;
            name
        }
        None => w.ask_name()?,
    };
    let description = w
        .ask(
            "description",
            Some("What this skill does and when to use it. Use when the user asks for X, mentions Y, or works on files matching Z."),
        )
        .unwrap_or_default();
    let allow = w
        .ask("allow (comma-separated tools, optional)", None)
        .unwrap_or_default();
    let paths = w
        .ask("paths (comma-separated globs, optional)", None)
        .unwrap_or_default();

    let pairs = vec![
        ("description".to_string(), description),
        ("allow".to_string(), allow),
        ("paths".to_string(), paths),
    ];
    write_item("skills", "skill", &name, &pairs)
}

fn wizard_command(name: Option<String>) -> Result<()> {
    let mut w = Wizard::new();
    let name = match name {
        Some(name) => {
            validate_name(&name)?;
            name
        }
        None => w.ask_name()?,
    };
    let description = w
        .ask(
            "description",
            Some("One sentence describing what this command does."),
        )
        .unwrap_or_default();
    let argument_hint = w
        .ask("argument-hint", Some("[your input]"))
        .unwrap_or_default();
    let agent = w
        .ask("agent (opencode agent, optional)", None)
        .unwrap_or_default();

    let pairs = vec![
        ("description".to_string(), description),
        ("argument-hint".to_string(), argument_hint),
        ("agent".to_string(), agent),
    ];
    write_item("commands", "command", &name, &pairs)
}

fn wizard_rule(name: Option<String>) -> Result<()> {
    let mut w = Wizard::new();
    let name = match name {
        Some(name) => {
            validate_name(&name)?;
            name
        }
        None => w.ask_name()?,
    };
    let description = w
        .ask("description", Some("What this rule covers."))
        .unwrap_or_default();
    let globs = w
        .ask("globs (comma-separated, empty = everywhere)", None)
        .unwrap_or_default();
    let always_apply = w.ask_yn("always-apply", false);

    let pairs = vec![
        ("description".to_string(), description),
        ("globs".to_string(), globs),
        (
            "always-apply".to_string(),
            if always_apply {
                "true".to_string()
            } else {
                String::new()
            },
        ),
    ];
    write_item("rules", "rule", &name, &pairs)
}

struct Wizard {
    reader: io::StdinLock<'static>,
    eof: bool,
}

impl Wizard {
    fn new() -> Self {
        Wizard {
            reader: io::stdin().lock(),
            eof: false,
        }
    }

    fn ask(&mut self, label: &str, default: Option<&str>) -> Option<String> {
        if self.eof {
            return default.map(|d| d.to_string());
        }
        match default {
            Some(d) => print!("{} [{}]: ", label, d),
            None => print!("{}: ", label),
        }
        let _ = io::stdout().flush();
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) | Err(_) => {
                self.eof = true;
                default.map(|d| d.to_string())
            }
            Ok(_) => {
                let value = line.trim().to_string();
                if value.is_empty() {
                    default.map(|d| d.to_string())
                } else {
                    Some(value)
                }
            }
        }
    }

    fn ask_name(&mut self) -> Result<String> {
        loop {
            match self.ask("name", None) {
                None => anyhow::bail!("name required (stdin closed)"),
                Some(name) if validate_name(&name).is_ok() => return Ok(name),
                Some(_) => {
                    println!(
                        "invalid name: use lowercase letters, digits and hyphens (e.g. code-review)"
                    );
                }
            }
        }
    }

    fn ask_f64(&mut self, label: &str) -> Result<Option<f64>> {
        loop {
            match self.ask(label, None) {
                None => return Ok(None),
                Some(v) if v.is_empty() => return Ok(None),
                Some(v) => match v.parse::<f64>() {
                    Ok(f) => return Ok(Some(f)),
                    Err(_) => println!("'{}' is not a number; press Enter to skip", v),
                },
            }
        }
    }

    fn ask_yn(&mut self, label: &str, default: bool) -> bool {
        match self.ask(label, Some(if default { "y" } else { "n" })) {
            Some(v) => v == "y" || v == "Y" || v == "yes" || v == "true",
            None => default,
        }
    }
}

fn write_item(dir: &str, kind: &str, name: &str, pairs: &[(String, String)]) -> Result<()> {
    ensure_ai()?;
    validate_name(name)?;
    let path = PathBuf::from(".ai").join(dir).join(format!("{}.md", name));
    if path.exists() {
        anyhow::bail!("{} '{}' already exists at {}", kind, name, path.display());
    }
    let scaffold = template::scaffold(dir, name)?;
    let body = frontmatter::parse(&scaffold)?.body;
    let mut content = write_frontmatter(pairs);
    if !body.is_empty() {
        content.push_str(&body);
        content.push('\n');
    }
    util::write(&path, &content)?;
    println!("Created {} '{}' at {}", kind, name, path.display());
    Ok(())
}

fn write_frontmatter(pairs: &[(String, String)]) -> String {
    let mut out = String::from("---\n");
    for (key, value) in pairs {
        if value.is_empty() {
            continue;
        }
        out.push_str(&format!("{}: {}\n", key, yaml_value(value)));
    }
    out.push_str("---\n");
    out
}

fn yaml_value(value: &str) -> String {
    if value.contains(": ")
        || value.contains(" #")
        || value.contains(['*', '{', '}', '[', ']', '#'])
    {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

fn create_ai_md() -> Result<()> {
    ensure_ai()?;
    let path = PathBuf::from(".ai").join("AI.md");
    if path.exists() {
        anyhow::bail!("{} already exists", path.display());
    }
    util::write(&path, template::DEFAULT_AI_MD)?;
    println!("Created {}", path.display());
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    const RESERVED: &[&str] = &[
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !RESERVED.contains(&name);
    if !ok {
        anyhow::bail!(
            "invalid name '{}': use lowercase letters, digits and hyphens (e.g. code-review)",
            name
        );
    }
    Ok(())
}
