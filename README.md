# dotai

Single source of truth for AI configs. Define your agents, skills, commands,
and rules once in `.ai/`, then generate identical setups for Claude Code
(`.claude/`), opencode (`.opencode/`), and Cursor (`.cursor/`) with one
command.

## How it works

```
.ai/                    <- source of truth (edit this)
  AI.md                 <- project instructions -> AGENTS.md
  agents/<name>.md      <- subagents
  skills/<name>/SKILL.md<- skills (+ supporting files)
  commands/<name>.md    <- slash commands
  rules/<name>.md       <- scoped rules
```

- `dotai new` — create `.ai/` from your template in `~/.config/dotai/template`
  (blank template is auto-created; override scaffolds with
  `template/<kind>/.scaffold.md`).
- `dotai new agent|skill|command|rule <name>` — scaffold a new item.
- `dotai gen [claude opencode cursor]` — regenerate everything. Uses the
  `providers` list in `~/.config/dotai/dotai.json` if no providers are given.
  Removes generated files whose `.ai/` item was deleted; your own files are
  never touched.
- `dotai clear` — remove AGENTS.md and all generated provider files
  (confirms first; `.ai/` is kept).

## Install

Published on [crates.io](https://crates.io/crates/dotai). Requires Rust
(cargo); installs to `~/.cargo/bin` (already on your PATH with rustup):

```sh
cargo install dotai
```

Building from source: `cargo install --path .`. Custom location:
`cargo install --path . --root ~/.local` (installs to `~/.local/bin`). Or use
the helper scripts, which also check PATH:

| OS | Command |
| --- | --- |
| macOS / Linux | `./scripts/install.sh` (default `~/.local/bin`; `DOTAI_INSTALL_DIR` overrides) |
| Windows | `.\scripts\install.ps1` (PowerShell; default `~\.local\bin`; `DOTAI_INSTALL_DIR` overrides) |

## Quick start

```sh
dotai new
dotai new agent code-review
dotai new skill deploy
dotai gen
```

Edit files in `.ai/`, run `dotai gen` again to sync — that's the whole loop.
