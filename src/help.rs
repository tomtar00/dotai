pub fn overview() {
    println!(
        "dotai new <kind> — create items in .ai/. Run `dotai new <kind> help` for the\n\
         frontmatter parameters of each kind.\n\
         \n\
         Kinds:\n\
         \x20 agent    subagents (model, effort, allow, deny, ...)\n\
         \x20 skill    reusable skill instructions (+ supporting files)\n\
         \x20 command  slash commands with {{input}} / {{1}} arguments\n\
         \x20 rule     scoped rules for matching files\n\
         \x20 ai       create .ai/AI.md if missing\n\
         \n\
         Everything you can set via prompts or flags can also be written\n\
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
    println!(
        "Agent frontmatter (.ai/agents/<name>.md)\n\
         \n\
         name          Required by the filename: lowercase letters, digits, hyphens.\n\
         description   What this agent does and when to delegate to it. The main\n\
                       agent reads this to decide when to use it.\n\
         model         small | medium | large — mapped per provider in dotai.json\n\
                       (default medium), or a full model id such as gpt-5.\n\
         effort        low | medium | high | xhigh | max — reasoning effort\n\
                       (default medium). Claude Code: `effort:`; opencode:\n\
                       `reasoningEffort:`; Cursor: model[effort=...].\n\
         temperature   Optional number (e.g. 0.1). Lower is more deterministic.\n\
         mode          subagent | primary (opencode only; default subagent).\n\
         allow         Comma-separated tools this agent may use, e.g.\n\
                       read, grep, glob, list, edit, bash, webfetch, websearch, task.\n\
                       Maps to Claude `tools:` and the opencode permission map.\n\
         deny          Comma-separated tools this agent must not use. Maps to\n\
                       Claude `disallowed-tools:` and opencode `permission:`\n\
                       denials. If it includes `edit`, Cursor gets readonly: true.\n\
         \n\
         Example:\n\
         ---\n\
         description: Reviews code for correctness and security.\n\
         model: medium\n\
         effort: high\n\
         allow: read, grep, glob, bash\n\
         deny: edit\n\
         ---"
    );
}

pub fn skill() {
    println!(
        "Skill frontmatter (.ai/skills/<name>/SKILL.md)\n\
         \n\
         name          Required by the directory name: lowercase letters, digits,\n\
                       hyphens.\n\
         description   What this skill does and when to use it. The model reads\n\
                       this to decide when to invoke the skill.\n\
         allow         Optional comma-separated tools the skill may use, e.g.\n\
                       read, grep, glob, bash. Maps to Claude `allowed-tools:`.\n\
         paths         Optional comma-separated globs of files this skill works\n\
                       with (Cursor).\n\
         \n\
         Supporting files (scripts, references) live next to SKILL.md and are\n\
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
    println!(
        "Command frontmatter (.ai/commands/<name>.md)\n\
         \n\
         name            Required by the filename: lowercase letters, digits,\n\
                         hyphens.\n\
         description     What this slash command does.\n\
         argument-hint   Shown in the prompt as a placeholder for arguments, e.g.\n\
                         \"[env]\". Commands without it take no arguments.\n\
         agent           Optional: run this command as a specific agent\n\
                         (opencode).\n\
         \n\
         Use {{input}} for everything typed after the command, {{1}}/{{2}} for\n\
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
    println!(
        "Rule frontmatter (.ai/rules/<name>.md)\n\
         \n\
         name           Required by the filename: lowercase letters, digits,\n\
                        hyphens.\n\
         description    What this rule covers.\n\
         globs          Comma-separated glob patterns of files it applies to,\n\
                        e.g. **/*.ts,src/**/*.rs. Omit to apply everywhere.\n\
         always-apply   true | false — always active instead of only when the\n\
                        model opens matching files (Cursor).\n\
         \n\
         Example:\n\
         ---\n\
         description: Commit message style.\n\
         globs: \"**/*.{{ts,tsx}}\"\n\
         always-apply: false\n\
         ---"
    );
}
