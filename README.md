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
dotai new agent code-review      # interactive wizard for the rest
dotai new agent architect --model large --effort xhigh   # flags skip prompts
dotai gen                        # generate AGENTS.md, .claude/, .cursor/, .opencode/
dotai gen --force                # overwrite files that existed before dotai
dotai clear                      # remove everything dotai generated
dotai config                     # edit dotai.json in $DOTAI_EDITOR / $VISUAL / $EDITOR / vi
dotai verify                     # check syntax of everything in .ai/
```

Edit files in `.ai/` and run `dotai gen` again — only changed files are
updated, and files that pre-date dotai are never touched.

## Commands

| Command | What it does |
| --- | --- |
| `dotai new` | Create `.ai/` from your template |
| `dotai new agent\|skill\|command\|rule [name]` | Scaffold an item; prompts for anything not given as a flag (`Enter` = default, empty on optional fields omits them) |
| `dotai new agent\|skill\|command\|rule help` | Print the frontmatter reference for that kind (nothing is created) |
| `dotai gen [providers...]` | Generate configs from `.ai/` (default: providers in dotai.json) |
| `dotai gen --force` | Overwrite files that existed before dotai managed them |
| `dotai providers` | List available and configured providers |
| `dotai clear` | Remove AGENTS.md and everything dotai generated |
| `dotai config` | Open dotai.json in `$DOTAI_EDITOR`, `$VISUAL`, `$EDITOR`, or `vi`/`notepad` |
| `dotai verify` | Check syntax of all agent, skill, command and rule files in `.ai/`; exits non-zero on problems |

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
as-is), plus `allow:` and `deny:` tool lists — e.g.
`allow: read, grep, glob, list` and `deny: edit`. Run
`dotai new agent help` for the full reference of every kind.

## Config

`~/.config/dotai/dotai.json` is auto-created and editable (`dotai config`):

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
