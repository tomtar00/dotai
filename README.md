# dotai

Single source of truth for AI configs. Define agents, skills, commands and
rules once in `.ai/`, then generate matching configs for Claude Code, Cursor
and opencode with one command.

## Install

```sh
cargo install dotai
```

Published on [crates.io](https://crates.io/crates/dotai). Requires Rust.

## Quick start

```sh
dotai new                        # create .ai/ from your template
dotai new agent code-review      # scaffold an agent
dotai new agent architect --model large --effort xhigh
dotai gen                        # generate AGENTS.md, .claude/, .cursor/, .opencode/
dotai gen --force                # overwrite files that existed before dotai
dotai clear                      # remove everything dotai generated
```

Edit files in `.ai/` and run `dotai gen` again — only changed files are
updated, and files that pre-date dotai are never touched.

## Layout

```
.ai/
  AI.md                  -> AGENTS.md
  agents/<name>.md       -> subagents
  skills/<name>/SKILL.md -> skills (+ supporting files)
  commands/<name>.md     -> slash commands
  rules/<name>.md        -> scoped rules
  manifest.json          -> tracks what dotai generated (don't edit)
```

Agent frontmatter: `model: small|medium|large` and
`effort: low|medium|high|xhigh|max` (or a full model name like `gpt-5`, used
as-is).

## Config

`~/.config/dotai/dotai.json` is auto-created and editable:

```json
{
  "providers": ["claude", "opencode", "cursor"],
  "models": {
    "claude":   { "small": "claude-haiku-4.5", "medium": "claude-sonnet-5", "large": "claude-opus-5" },
    "opencode": { "small": "anthropic/claude-haiku-4.5", "medium": "anthropic/claude-sonnet-5", "large": "anthropic/claude-opus-5" },
    "cursor":   { "small": "claude-haiku-4.5", "medium": "claude-sonnet-5", "large": "claude-opus-5" }
  }
}
```

- `providers` — which configs `dotai gen` generates by default.
- `models` — how agent model sizes map to concrete models per provider.

Customize the `.ai/` template (including per-kind scaffolds in
`template/<kind>/.scaffold.md`) under `~/.config/dotai/template`.
