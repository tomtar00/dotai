use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_dotai");

struct TestEnv {
    _home: TempDir,
    _proj: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        Self {
            _home: tempfile::tempdir().unwrap(),
            _proj: tempfile::tempdir().unwrap(),
        }
    }

    fn home(&self) -> &Path {
        self._home.path()
    }

    fn proj(&self) -> &Path {
        self._proj.path()
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .env("HOME", self.home())
            .current_dir(self.proj())
            .output()
            .unwrap()
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        let mut child = Command::new(BIN)
            .args(args)
            .env("HOME", self.home())
            .current_dir(self.proj())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn run_with(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(BIN);
        cmd.args(args)
            .env("HOME", self.home())
            .current_dir(self.proj());
        for (key, value) in envs {
            cmd.env(key, value);
        }
        cmd.output().unwrap()
    }

    fn ok(&self, args: &[&str]) {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "`dotai {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn fails(&self, args: &[&str], contains: &str) {
        let out = self.run(args);
        assert!(
            !out.status.success(),
            "`dotai {}` should have failed",
            args.join(" ")
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(contains),
            "stderr should contain {:?}, got: {}",
            contains,
            stderr
        );
    }

    fn stdout(&self, args: &[&str]) -> String {
        String::from_utf8_lossy(&self.run(args).stdout).to_string()
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.proj().join(rel)
    }

    fn exists(&self, rel: &str) -> bool {
        self.path(rel).exists()
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).unwrap_or_else(|e| panic!("read {}: {}", rel, e))
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn write_home(&self, rel: &str, content: &str) {
        let path = self.home().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path(rel)).unwrap();
    }

    fn remove_all(&self, rel: &str) {
        std::fs::remove_dir_all(self.path(rel)).unwrap();
    }

    fn setup_full(&self) {
        self.ok(&["new"]);
        self.ok(&["new", "agent", "code-review"]);
        self.ok(&["new", "skill", "summarize"]);
        self.ok(&["new", "command", "release"]);
        self.ok(&["new", "rule", "commit-style"]);
    }
}

fn setup() -> TestEnv {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env
}

#[test]
fn new_bootstraps_template_and_config() {
    let env = TestEnv::new();
    env.ok(&["new"]);

    for sub in ["rules", "agents", "skills", "commands"] {
        assert!(env.exists(&format!(".ai/{}", sub)), ".ai/{} missing", sub);
    }
    assert!(env.exists(".ai/AI.md"));

    let home = env.home();
    assert!(home.join(".config/dotai/template/AI.md").exists());
    assert!(home.join(".config/dotai/dotai.json").exists());
    let config = std::fs::read_to_string(home.join(".config/dotai/dotai.json")).unwrap();
    for provider in ["claude", "opencode", "cursor"] {
        assert!(config.contains(provider), "config missing {}", provider);
    }
    assert!(
        config.contains("\"models\""),
        "config missing models: {}",
        config
    );

    let providers = env.stdout(&["providers"]);
    assert!(providers.contains("claude, opencode, cursor"));

    env.fails(&["new"], ".ai directory already exists");
}

#[test]
fn new_scaffolds_items_and_validates_names() {
    let env = setup();
    env.ok(&["new", "agent", "code-review"]);
    env.ok(&["new", "skill", "summarize"]);
    env.ok(&["new", "command", "release"]);
    env.ok(&["new", "rule", "commit-style"]);
    env.remove(".ai/AI.md");
    env.ok(&["new", "ai"]);
    assert!(env.exists(".ai/AI.md"));

    let agent = env.read(".ai/agents/code-review.md");
    assert!(
        agent.contains("description:"),
        "agent scaffold missing description"
    );
    assert!(agent.contains("allow: read, grep, glob, list"));
    assert!(
        agent.contains("code-review"),
        "scaffold name not substituted"
    );

    let skill = env.read(".ai/skills/summarize.md");
    assert!(skill.contains("description:"));
    let command = env.read(".ai/commands/release.md");
    assert!(command.contains("argument-hint:"));
    let rule = env.read(".ai/rules/commit-style.md");
    assert!(rule.contains("description: What this rule covers."));

    env.fails(&["new", "rule", "commit-style"], "already exists");
    env.fails(&["new", "agent", "Bad Name"], "invalid name");
    env.fails(&["new", "agent", "bad_name"], "invalid name");
    env.fails(&["new", "agent", "con"], "invalid name");
    env.fails(&["new", "agent", "COM1"], "invalid name");
}

#[test]
fn new_kind_help_prints_reference_without_side_effects() {
    let env = TestEnv::new();
    for (kind, expected) in [
        ("agent", "Agent frontmatter"),
        ("skill", "Skill frontmatter"),
        ("command", "Command frontmatter"),
        ("rule", "Rule frontmatter"),
    ] {
        let out = env.stdout(&["new", kind, "help"]);
        assert!(out.contains(expected), "{} help missing {}", kind, expected);
    }
    assert!(!env.exists(".ai"), "help must not bootstrap .ai");
    assert!(
        !env.home().join(".config/dotai/dotai.json").exists(),
        "help must not create the config"
    );

    let overview = env.stdout(&["new", "help"]);
    assert!(overview.contains("Kinds:"), "overview: {}", overview);
    assert!(
        overview.contains("agent"),
        "overview must list kinds: {}",
        overview
    );

    let unknown = env.stdout(&["new", "help", "bogus"]);
    assert!(unknown.contains("Unknown kind"));
}

#[test]
fn wizard_prompts_and_writes_all_values() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    let out = env.run_stdin(
        &["new", "agent", "wiz-agt"],
        "A smart reviewer.\nlarge\nhigh\n0.5\nprimary\nread, bash\nedit\n",
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let agent = env.read(".ai/agents/wiz-agt.md");
    assert!(
        agent.contains("description: A smart reviewer."),
        "{}",
        agent
    );
    assert!(agent.contains("model: large"), "{}", agent);
    assert!(agent.contains("effort: high"), "{}", agent);
    assert!(agent.contains("temperature: 0.5"), "{}", agent);
    assert!(agent.contains("mode: primary"), "{}", agent);
    assert!(agent.contains("allow: read, bash"), "{}", agent);
    assert!(agent.contains("deny: edit"), "{}", agent);
}

#[test]
fn wizard_defaults_on_empty_and_eof() {
    let env = TestEnv::new();
    env.ok(&["new"]);

    let out = env.run_stdin(&["new", "agent", "eof-agt"], "");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let agent = env.read(".ai/agents/eof-agt.md");
    assert!(
        agent.contains("description: One sentence on when to delegate to this agent."),
        "{}",
        agent
    );
    assert!(agent.contains("model: medium"), "{}", agent);
    assert!(agent.contains("effort: medium"), "{}", agent);
    assert!(agent.contains("mode: subagent"), "{}", agent);
    assert!(agent.contains("allow: read, grep, glob, list"), "{}", agent);
    assert!(!agent.contains("temperature:"), "{}", agent);
    assert!(!agent.contains("deny:"), "{}", agent);

    let out = env.run_stdin(&["new", "agent", "empty-agt"], "\n\n\n\n\n\n\n");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let agent = env.read(".ai/agents/empty-agt.md");
    assert!(agent.contains("model: medium"), "{}", agent);
    assert!(!agent.contains("temperature:"), "{}", agent);
    assert!(!agent.contains("deny:"), "{}", agent);
}

#[test]
fn wizard_flags_skip_prompts_and_name_is_prompted() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    let out = env.run_stdin(
        &["new", "agent", "--model", "large", "--effort", "xhigh"],
        "named-agent\nDescription here.\n0.2\n\nread, bash\n\n",
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let agent = env.read(".ai/agents/named-agent.md");
    assert!(agent.contains("model: large"), "{}", agent);
    assert!(agent.contains("effort: xhigh"), "{}", agent);
    assert!(
        agent.contains("description: Description here."),
        "{}",
        agent
    );
    assert!(agent.contains("temperature: 0.2"), "{}", agent);
    assert!(agent.contains("mode: subagent"), "{}", agent);
    assert!(agent.contains("allow: read, bash"), "{}", agent);
    assert!(!agent.contains("deny:"), "{}", agent);
}

#[test]
fn config_command_opens_editor_and_creates_config() {
    let env = TestEnv::new();
    let out = env.run_with(&["config"], &[("DOTAI_EDITOR", "echo edited")]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("edited"),
        "editor must be invoked with the config path"
    );
    let config_path = env.home().join(".config/dotai/dotai.json");
    assert!(config_path.exists(), "config must be created");
    let config = std::fs::read_to_string(config_path).unwrap();
    assert!(config.contains("\"providers\""), "{}", config);
}

#[test]
fn config_command_fails_when_editor_fails() {
    let env = TestEnv::new();
    let out = env.run_with(&["config"], &[("DOTAI_EDITOR", "false")]);
    assert!(
        !out.status.success(),
        "non-zero editor exit must fail the command"
    );
}

#[test]
fn gen_requires_ai_dir_and_creates_config() {
    let env = TestEnv::new();
    env.fails(
        &["gen", "claude"],
        ".ai directory does not exist. Run `dotai new` first",
    );

    env.write(".ai/AI.md", "# Project\n");
    env.ok(&["gen"]); // no config -> auto-created defaults
    assert!(
        env.home().join(".config/dotai/dotai.json").exists(),
        "gen must create the default config"
    );
    env.fails(
        &["gen", "bogus"],
        "unknown provider 'bogus'. Known providers: claude, cursor, opencode",
    );
}

#[test]
fn gen_writes_correct_provider_syntax() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: Reviews code.\nallow: read, grep, glob, list\ndeny: edit, bash\n---\nReview carefully.\n",
    );
    env.write(
        ".ai/agents/impl.md",
        "---\ndescription: Writes code.\ntools: read, bash\nmodel: anthropic/claude-sonnet-4-5\n---\nImplement stuff.\n",
    );
    env.write(
        ".ai/skills/summarize/SKILL.md",
        "---\ndescription: Summarizes files.\nallowed-tools: read, grep\n---\nSummarize the input.\n",
    );
    env.write(
        ".ai/commands/release.md",
        "---\ndescription: Deploy a release.\nargument-hint: [env]\n---\nDeploy {{1}} to {{2}} using {{input}}. Check {{issue}}.\n",
    );
    env.write(
        ".ai/rules/commit-style.md",
        "---\ndescription: Commit style.\nglobs: \"**/*.{ts,tsx}\"\nalways-apply: true\n---\nUse conventional commits.\n",
    );

    env.ok(&["gen", "claude", "opencode", "cursor"]);

    let ai_md = env.read(".ai/AI.md");
    assert!(env.read("AGENTS.md").starts_with(&ai_md));

    let claude_md = env.read(".claude/CLAUDE.md");
    assert!(
        claude_md.contains("@../AGENTS.md"),
        "CLAUDE.md must import root AGENTS.md via ../"
    );

    let claude_agent = env.read(".claude/agents/code-review.md");
    assert!(claude_agent.contains("name: code-review"));
    assert!(
        claude_agent.contains("tools: Read, Grep, Glob"),
        "claude tools must be Title-cased and deduped"
    );
    assert!(
        claude_agent.contains("disallowed-tools: Edit, Bash"),
        "claude deny must map to disallowed-tools"
    );

    let claude_impl = env.read(".claude/agents/impl.md");
    assert!(
        claude_impl.contains("model: claude-sonnet-4-5"),
        "anthropic/ prefix must be stripped"
    );

    let claude_skill = env.read(".claude/skills/summarize/SKILL.md");
    assert!(claude_skill.contains("allowed-tools: Read, Grep"));

    let claude_cmd = env.read(".claude/commands/release.md");
    assert!(
        claude_cmd.contains("arguments: [issue]"),
        "named args must be declared: {}",
        claude_cmd
    );
    assert!(
        claude_cmd.contains("Deploy $0 to $1 using $ARGUMENTS. Check $issue."),
        "claude args are 0-based"
    );

    let opencode_agent = env.read(".opencode/agents/code-review.md");
    assert!(
        !opencode_agent.contains("tools:"),
        "opencode tools field is deprecated: {}",
        opencode_agent
    );
    assert!(opencode_agent.contains("mode: subagent"));
    assert!(opencode_agent.contains("permission:"));
    assert!(opencode_agent.contains("edit: deny"));
    assert!(opencode_agent.contains("bash: deny"));

    let opencode_impl = env.read(".opencode/agents/impl.md");
    assert!(
        opencode_impl.contains("model: anthropic/claude-sonnet-4-5"),
        "opencode model keeps provider prefix"
    );
    assert!(
        opencode_impl.contains("bash: allow"),
        "opencode allow list must map to permission entries: {}",
        opencode_impl
    );

    let opencode_cmd = env.read(".opencode/commands/release.md");
    assert!(
        opencode_cmd.contains("Deploy $1 to $2 using $ARGUMENTS. Check {{issue}}."),
        "opencode args are 1-based"
    );

    let cursor_rule = env.read(".cursor/rules/commit-style.mdc");
    assert!(
        cursor_rule.contains("globs:"),
        "cursor rule needs globs frontmatter"
    );
    assert!(
        cursor_rule.contains("\n  - \"**/*.{ts,tsx}\""),
        "cursor globs must be a YAML list: {}",
        cursor_rule
    );
    assert!(cursor_rule.contains("alwaysApply: true"));

    let cursor_agent = env.read(".cursor/agents/code-review.md");
    assert!(
        cursor_agent.contains("readonly: true"),
        "readonly tools must map to readonly flag"
    );
    let cursor_impl = env.read(".cursor/agents/impl.md");
    assert!(!cursor_impl.contains("readonly: true"));

    let cursor_cmd = env.read(".cursor/commands/release.md");
    assert!(
        cursor_cmd.contains("Deploy {{1}} to {{2}} using {{input}}. Check {{issue}}."),
        "cursor keeps {{}} vars"
    );

    for provider in ["claude", "opencode", "cursor"] {
        assert!(env.exists(&format!(".{}/skills/summarize/SKILL.md", provider)));
    }
}

#[test]
fn modify_and_remove_syncs_provider_files() {
    let env = TestEnv::new();
    env.setup_full();
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    assert!(env.exists(".claude/commands/release.md"));
    assert!(env.exists(".cursor/rules/commit-style.mdc"));

    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: New description.\ntools: read\n---\nUpdated body here.\n",
    );
    env.ok(&["gen", "claude"]);
    let agent = env.read(".claude/agents/code-review.md");
    assert!(agent.contains("Updated body here."));
    assert!(agent.contains("description: New description."));
    assert!(agent.contains("tools: Read"), "tools must reflect the edit");

    env.remove(".ai/commands/release.md");
    env.remove(".ai/rules/commit-style.md");
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    assert!(!env.exists(".claude/commands/release.md"));
    assert!(!env.exists(".opencode/commands/release.md"));
    assert!(!env.exists(".cursor/commands/release.md"));
    assert!(!env.exists(".claude/rules/commit-style.md"));
    assert!(!env.exists(".cursor/rules/commit-style.mdc"));
    assert!(
        !env.exists(".claude/commands"),
        "empty commands dir must be pruned"
    );
    assert!(
        env.exists(".claude/agents/code-review.md"),
        "remaining items must survive"
    );

    env.remove_all(".ai/agents");
    env.ok(&["gen", "claude"]);
    assert!(
        !env.exists(".claude/agents"),
        "empty agents dir must be pruned"
    );
    assert!(env.exists(".claude/CLAUDE.md"));
}

#[test]
fn skill_resources_sync() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env.write(
        ".ai/skills/toolkit/SKILL.md",
        "---\ndescription: Runs the toolkit.\n---\nUse the scripts.\n",
    );
    env.write(".ai/skills/toolkit/README.md", "docs\n");
    env.write(".ai/skills/toolkit/scripts/run.sh", "#!/bin/sh\necho hi\n");

    env.ok(&["gen", "claude", "opencode", "cursor"]);
    for provider in ["claude", "opencode", "cursor"] {
        assert!(env.exists(&format!(".{}/skills/toolkit/scripts/run.sh", provider)));
        assert!(env.exists(&format!(".{}/skills/toolkit/README.md", provider)));
    }

    env.remove(".ai/skills/toolkit/README.md");
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    for provider in ["claude", "opencode", "cursor"] {
        assert!(
            !env.exists(&format!(".{}/skills/toolkit/README.md", provider)),
            "{} stale resource kept",
            provider
        );
        assert!(env.exists(&format!(".{}/skills/toolkit/scripts/run.sh", provider)));
    }

    env.remove(".ai/skills/toolkit/scripts/run.sh");
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    for provider in ["claude", "opencode", "cursor"] {
        assert!(
            !env.exists(&format!(".{}/skills/toolkit/scripts", provider)),
            "{} empty resource dir kept",
            provider
        );
    }

    env.remove_all(".ai/skills/toolkit");
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    for provider in ["claude", "opencode", "cursor"] {
        assert!(
            !env.exists(&format!(".{}/skills/toolkit", provider)),
            "{} stale skill dir kept",
            provider
        );
        assert!(
            !env.exists(&format!(".{}/skills", provider)),
            "{} empty skills dir kept",
            provider
        );
    }
}

#[test]
fn user_files_are_never_touched() {
    let env = TestEnv::new();
    env.setup_full();
    env.write(".cursor/mcp.json", "{}\n");
    env.write(
        ".cursor/commands/manual.md",
        "# my own command\nno marker here\n",
    );
    env.write(".claude/settings.json", "{}\n");
    env.write(
        ".claude/commands/manual.md",
        "# my own claude command\nno marker here\n",
    );

    env.ok(&["gen", "claude", "opencode", "cursor"]);

    assert!(env.exists(".cursor/mcp.json"));
    assert!(env.exists(".claude/settings.json"));
    assert_eq!(
        env.read(".cursor/commands/manual.md"),
        "# my own command\nno marker here\n"
    );
    assert_eq!(
        env.read(".claude/commands/manual.md"),
        "# my own claude command\nno marker here\n"
    );

    env.remove(".ai/commands/release.md");
    env.ok(&["gen", "claude", "cursor"]);
    assert!(
        !env.exists(".cursor/commands/release.md"),
        "generated command must go"
    );
    assert!(
        env.exists(".cursor/commands/manual.md"),
        "manual command must stay"
    );
    assert!(
        env.exists(".claude/commands/manual.md"),
        "manual claude command must stay"
    );
}

#[test]
fn gen_is_idempotent() {
    let env = TestEnv::new();
    env.setup_full();
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    let before = env.read(".claude/agents/code-review.md");
    let files_before = count_files(&env.path(""));
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    let after = env.read(".claude/agents/code-review.md");
    let files_after = count_files(&env.path(""));
    assert_eq!(before, after);
    assert_eq!(
        files_before, files_after,
        "regeneration must not add or remove files"
    );
}

#[test]
fn rename_syncs_across_providers() {
    let env = TestEnv::new();
    env.setup_full();
    env.ok(&["gen", "claude", "opencode", "cursor"]);

    let content = env.read(".ai/agents/code-review.md");
    env.write(".ai/agents/reviewer.md", &content);
    env.remove(".ai/agents/code-review.md");
    env.ok(&["gen", "claude", "opencode", "cursor"]);

    for provider in ["claude", "opencode", "cursor"] {
        assert!(!env.exists(&format!(".{}/agents/code-review.md", provider)));
        assert!(env.exists(&format!(".{}/agents/reviewer.md", provider)));
    }
}

#[test]
fn clear_removes_generated_files() {
    let env = TestEnv::new();
    env.setup_full();
    env.write(".cursor/mcp.json", "{}\n");
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    assert!(env.exists("AGENTS.md"));
    assert!(env.exists(".claude"));
    assert!(env.exists(".opencode"));
    assert!(env.exists(".cursor"));

    env.ok(&["clear"]); // "Aborted." on non-y input, nothing removed
    assert!(env.exists("AGENTS.md"), "declining clear must keep files");
    assert!(env.exists(".claude"));

    let out = env.run_stdin(&["clear"], "y\n");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!env.exists("AGENTS.md"));
    assert!(!env.exists(".claude"));
    assert!(!env.exists(".opencode"));
    assert!(
        env.exists(".cursor"),
        ".cursor must stay while it holds user files"
    );
    assert!(
        env.exists(".cursor/mcp.json"),
        "user files must survive clear"
    );
    let cursor_entries: Vec<_> = std::fs::read_dir(env.path(".cursor")).unwrap().collect();
    assert_eq!(
        cursor_entries.len(),
        1,
        "only mcp.json should remain in .cursor"
    );
    assert!(env.exists(".ai"), "source of truth must survive clear");

    let out = env.run_stdin(&["clear"], "y\n");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Removed 0 generated file(s)."),
        "second clear should be a no-op: {}",
        stdout
    );
}

#[test]
fn custom_scaffold_override() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    let template = env
        .home()
        .join(".config/dotai/template/agents/.scaffold.md");
    std::fs::write(&template, "CUSTOM AGENT: {name}\n").unwrap();

    env.ok(&["new", "agent", "custom-one"]);
    let content = env.read(".ai/agents/custom-one.md");
    assert!(
        content.contains("CUSTOM AGENT: custom-one"),
        "override scaffold must be used: {}",
        content
    );
}

fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                count += count_files(&entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                out.extend(walk_files(&entry.path()));
            } else {
                out.push(entry.path());
            }
        }
    }
    out
}

#[test]
fn gen_auto_creates_default_config() {
    let env = TestEnv::new();
    env.write(".ai/AI.md", "# Project\n");
    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: Reviews.\n---\nBody.\n",
    );

    env.ok(&["gen"]);

    let config = std::fs::read_to_string(env.home().join(".config/dotai/dotai.json")).unwrap();
    assert!(config.contains("\"models\""), "missing models: {}", config);
    assert!(
        config.contains("claude-opus-5"),
        "missing new models: {}",
        config
    );
    assert!(
        config.contains("anthropic/claude-sonnet-5"),
        "missing prefixed models: {}",
        config
    );
    assert!(env.exists(".claude/agents/code-review.md"));
    assert!(env.exists("AGENTS.md"));
}

#[test]
fn preexisting_files_are_never_overwritten() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env.write(
        ".claude/agents/code-review.md",
        "# MY OWN AGENT\ncustom content\n",
    );
    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: Mine.\n---\nBody.\n",
    );

    env.ok(&["gen", "claude"]);
    assert_eq!(
        env.read(".claude/agents/code-review.md"),
        "# MY OWN AGENT\ncustom content\n",
        "pre-existing file must never be overwritten"
    );
    let out = env.stdout(&["gen", "claude"]);
    assert!(
        out.contains("pre-existing"),
        "gen should warn about skipped files: {}",
        out
    );

    env.ok(&["gen", "claude", "--force"]);
    let agent = env.read(".claude/agents/code-review.md");
    assert!(agent.contains("Body."), "--force must overwrite: {}", agent);
    assert!(agent.contains("name: code-review"));
}

#[test]
fn gen_updates_only_changed_files() {
    let env = TestEnv::new();
    env.setup_full();
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    let skill_before = env.read(".claude/skills/summarize/SKILL.md");
    assert!(skill_before.contains("summarize"));

    env.write(".claude/agents/code-review.md", "HAND EDIT\n");
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    assert_eq!(
        env.read(".claude/agents/code-review.md"),
        "HAND EDIT\n",
        "hand edit must survive when the .ai source is unchanged"
    );
    assert_eq!(
        env.read(".claude/skills/summarize/SKILL.md"),
        skill_before,
        "unchanged items must not be rewritten"
    );

    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: New.\n---\nNew body.\n",
    );
    env.ok(&["gen", "claude"]);
    let agent = env.read(".claude/agents/code-review.md");
    assert!(agent.contains("New body."), "source change must regenerate");
    assert!(!agent.contains("HAND EDIT"));

    let manifest = env.read(".ai/manifest.json");
    assert!(
        manifest.contains(".claude/agents/code-review.md"),
        "manifest must track generated files: {}",
        manifest
    );
}

#[test]
fn agent_model_and_effort_map_per_provider() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env.ok(&[
        "new", "agent", "reviewer", "--model", "large", "--effort", "xhigh",
    ]);
    env.ok(&["new", "agent", "writer"]);

    let reviewer = env.read(".ai/agents/reviewer.md");
    assert!(reviewer.contains("model: large"), "{}", reviewer);
    assert!(reviewer.contains("effort: xhigh"), "{}", reviewer);
    let writer = env.read(".ai/agents/writer.md");
    assert!(writer.contains("model: medium"), "{}", writer);
    assert!(writer.contains("effort: medium"), "{}", writer);

    assert!(!env
        .run(&["new", "agent", "bad", "--model", "huge"])
        .status
        .success());
    assert!(!env
        .run(&["new", "agent", "bad", "--effort", "extreme"])
        .status
        .success());

    env.ok(&["gen", "claude", "opencode", "cursor"]);

    let claude_reviewer = env.read(".claude/agents/reviewer.md");
    assert!(
        claude_reviewer.contains("model: claude-opus-5"),
        "{}",
        claude_reviewer
    );
    assert!(
        claude_reviewer.contains("effort: xhigh"),
        "{}",
        claude_reviewer
    );
    let opencode_reviewer = env.read(".opencode/agents/reviewer.md");
    assert!(opencode_reviewer.contains("model: anthropic/claude-opus-5"));
    assert!(opencode_reviewer.contains("reasoningEffort: xhigh"));
    let cursor_reviewer = env.read(".cursor/agents/reviewer.md");
    assert!(
        cursor_reviewer.contains("model: claude-opus-5[effort=xhigh]"),
        "{}",
        cursor_reviewer
    );

    let claude_writer = env.read(".claude/agents/writer.md");
    assert!(
        claude_writer.contains("model: claude-sonnet-5"),
        "{}",
        claude_writer
    );
    assert!(claude_writer.contains("effort: medium"));
    let opencode_writer = env.read(".opencode/agents/writer.md");
    assert!(opencode_writer.contains("reasoningEffort: medium"));
    let cursor_writer = env.read(".cursor/agents/writer.md");
    assert!(cursor_writer.contains("model: claude-sonnet-5[effort=medium]"));

    env.write_home(
        ".config/dotai/dotai.json",
        r#"{"providers": ["claude"], "models": {"claude": {"medium": "my-medium-model"}}}"#,
    );
    env.ok(&["gen", "claude"]);
    let writer = env.read(".claude/agents/writer.md");
    assert!(writer.contains("model: my-medium-model"), "{}", writer);
    assert!(writer.contains("effort: medium"));
}

#[test]
fn config_without_models_still_loads() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env.write_home(".config/dotai/dotai.json", r#"{"providers": ["claude"]}"#);
    env.ok(&["new", "agent", "foo"]);
    env.ok(&["gen"]);
    let agent = env.read(".claude/agents/foo.md");
    assert!(
        !agent.contains("model:"),
        "unmapped level must not emit a model: {}",
        agent
    );
    assert!(agent.contains("effort: medium"), "{}", agent);
}

#[test]
fn generated_files_have_no_marker() {
    let env = TestEnv::new();
    env.setup_full();
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    for path in walk_files(&env.path("")) {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !content.contains("Generated by dotai"),
            "{} still carries the marker",
            path.display()
        );
    }
}

#[test]
fn migrates_legacy_marker_files() {
    let env = TestEnv::new();
    env.ok(&["new"]);
    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: Mine.\n---\nBody.\n",
    );
    let legacy = "<!-- Generated by dotai from .ai/. Do not edit. Run `dotai gen` to regenerate. -->\n---\nname: code-review\ndescription: Mine.\n---\nBody.\n";
    env.write(".claude/agents/code-review.md", legacy);
    env.write(
        ".claude/commands/old.md",
        "<!-- Generated by dotai from .ai/. Do not edit. Run `dotai gen` to regenerate. -->\nold\n",
    );

    env.ok(&["gen", "claude"]);

    let adopted = env.read(".claude/agents/code-review.md");
    assert!(
        !adopted.contains("Generated by dotai"),
        "adopted file must be marker-free: {}",
        adopted
    );
    assert!(adopted.contains("Body."));
    assert!(
        !env.exists(".claude/commands/old.md"),
        "orphan legacy file must be removed"
    );

    env.write(
        ".ai/agents/code-review.md",
        "---\ndescription: Mine.\n---\nBody 2.\n",
    );
    env.ok(&["gen", "claude"]);
    assert!(
        env.read(".claude/agents/code-review.md")
            .contains("Body 2."),
        "adopted file must be managed going forward"
    );

    let out = env.run_stdin(&["clear"], "y\n");
    assert!(out.status.success());
    assert!(!env.exists(".claude/agents/code-review.md"));
}

#[test]
fn clear_resets_manifest() {
    let env = TestEnv::new();
    env.setup_full();
    env.ok(&["gen", "claude", "opencode", "cursor"]);
    assert!(env.exists(".ai/manifest.json"));

    let out = env.run_stdin(&["clear"], "y\n");
    assert!(out.status.success());

    let manifest = std::fs::read_to_string(env.path(".ai/manifest.json")).unwrap();
    assert!(
        manifest.contains("\"files\": {}"),
        "manifest must be empty after clear: {}",
        manifest
    );

    env.ok(&["gen", "claude"]);
    assert!(env.exists(".claude/agents/code-review.md"));
}
