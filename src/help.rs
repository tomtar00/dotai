const DESC_W: usize = 60;

pub fn overview() {
    println!(
        "dotai new <kind> — create items in .ai/. Run `dotai new <kind> help`\n\
         for the frontmatter parameters of each kind.\n\
         \n\
         Kinds:\n"
    );
    println!(
        "{}",
        table(
            10,
            ("Kind", "What it creates"),
            &[
                (
                    "agent",
                    "Subagents: model, effort, temperature, mode, allow/deny tools."
                ),
                ("skill", "Reusable skills — SKILL.md plus supporting files."),
                ("command", "Slash commands using {input} and {1} arguments."),
                ("rule", "Scoped rules for matching files."),
                ("ai", "Create .ai/AI.md if missing."),
            ],
        )
    );
    println!(
        "\nEverything you can set via prompts or flags can also be written\n\
         directly in the markdown frontmatter of the generated file."
    );
}

pub fn item(kind: &str) {
    match kind {
        "agent" | "agents" => agent(),
        "skill" | "skills" => skill(),
        "command" | "commands" => command(),
        "rule" | "rules" => rule(),
        "" => overview(),
        _ => println!(
            "Unknown kind '{}'. Run `dotai new help` for the list.",
            kind
        ),
    }
}

pub fn agent() {
    println!("Agent frontmatter (.ai/agents/<name>.md)\n");
    println!(
        "{}",
        table(
            16,
            ("Parameter", "Description"),
            &[
                ("name", "Required by the filename: lowercase letters, digits and hyphens."),
                ("description", "What this agent does and when to delegate to it. The main agent reads this to decide when to use it."),
                ("model", "small, medium or large — mapped per provider in dotai.json (default medium), or a full model id such as gpt-5."),
                ("effort", "low, medium, high, xhigh or max — reasoning effort (default medium). Claude Code: `effort:`; opencode: `reasoningEffort:`; Cursor: model[effort=...]."),
                ("temperature", "Optional number, e.g. 0.1. Lower is more deterministic."),
                ("mode", "subagent or primary (opencode only; default subagent)."),
                ("allow", "Comma-separated tools this agent may use: read, grep, glob, list, edit, bash, webfetch, websearch, task. Maps to Claude `tools:` and the opencode permission map."),
                ("deny", "Comma-separated tools this agent must not use. Maps to Claude `disallowed-tools:` and opencode permission denials. If it includes `edit`, Cursor gets readonly: true."),
            ],
        )
    );
    println!(
        "\nExample:\n---\ndescription: Reviews code for correctness and security.\nmodel: medium\neffort: high\nallow: read, grep, glob, bash\ndeny: edit\n---"
    );
}

pub fn skill() {
    println!("Skill frontmatter (.ai/skills/<name>/SKILL.md)\n");
    println!(
        "{}",
        table(
            16,
            ("Parameter", "Description"),
            &[
                ("name", "Required by the directory name: lowercase letters, digits and hyphens."),
                ("description", "What this skill does and when to use it. The model reads this to decide when to invoke the skill."),
                ("allow", "Optional comma-separated tools the skill may use, e.g. read, grep, glob, bash. Maps to Claude `allowed-tools:`."),
                ("paths", "Optional comma-separated globs of files this skill works with (Cursor)."),
            ],
        )
    );
    println!(
        "\nSupporting files (scripts, references) live next to SKILL.md and are\n\
         copied alongside it.\n\
         \n\
         Example:\n\
         ---\n\
         description: Runs the project test suite.\n\
         allow: read, bash\n\
         ---"
    );
}

pub fn command() {
    println!("Command frontmatter (.ai/commands/<name>.md)\n");
    println!(
        "{}",
        table(
            16,
            ("Parameter", "Description"),
            &[
                ("name", "Required by the filename: lowercase letters, digits and hyphens."),
                ("description", "What this slash command does."),
                ("argument-hint", "Shown in the prompt as a placeholder for arguments, e.g. \"[env]\". Commands without it take no arguments."),
                ("agent", "Optional: run this command as a specific agent (opencode)."),
            ],
        )
    );
    println!(
        "\nUse {{input}} for everything typed after the command, {{1}} / {{2}} for\n\
         positional arguments, and {{name}} for named arguments.\n\
         \n\
         Example:\n\
         ---\n\
         description: Deploy a release.\n\
         argument-hint: [env]\n\
         ---\n\
         Deploy {{1}} to {{2}} using {{input}}."
    );
}

pub fn rule() {
    println!("Rule frontmatter (.ai/rules/<name>.md)\n");
    println!(
        "{}",
        table(
            16,
            ("Parameter", "Description"),
            &[
                ("name", "Required by the filename: lowercase letters, digits and hyphens."),
                ("description", "What this rule covers."),
                ("globs", "Comma-separated glob patterns of files it applies to, e.g. **/*.ts, src/**/*.rs. Omit to apply everywhere."),
                ("always-apply", "true or false — always active instead of only when the model opens matching files (Cursor)."),
            ],
        )
    );
    println!(
        "\nExample:\n---\ndescription: Commit message style.\nglobs: \"**/*.{{ts,tsx}}\"\nalways-apply: false\n---"
    );
}

fn table(col_w: usize, headers: (&str, &str), rows: &[(&str, &str)]) -> String {
    let top = format!("┌{}┬{}┐", "─".repeat(col_w + 2), "─".repeat(DESC_W + 2));
    let mid = format!("├{}┼{}┤", "─".repeat(col_w + 2), "─".repeat(DESC_W + 2));
    let bottom = format!("└{}┴{}┘", "─".repeat(col_w + 2), "─".repeat(DESC_W + 2));

    let mut out = String::new();
    out.push_str(&top);
    out.push('\n');
    out.push_str(&row(headers.0, headers.1, col_w));
    out.push('\n');
    out.push_str(&mid);
    out.push('\n');
    for (param, desc) in rows {
        let lines = wrap(desc, DESC_W);
        for (i, line) in lines.iter().enumerate() {
            out.push_str(&row(if i == 0 { param } else { "" }, line, col_w));
            out.push('\n');
        }
    }
    out.push_str(&bottom);
    out
}

fn row(param: &str, desc: &str, col_w: usize) -> String {
    format!("│ {param:<w$}│ {desc:<d$} │", w = col_w + 1, d = DESC_W)
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > width {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
