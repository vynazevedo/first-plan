# Adapter Templates

Tool-specific templates that render `.first-plan/` IR into per-tool instruction files.

**Actual templates live in `engine/crates/core/adapters/`** (required for `include_str!` at compile time). This directory exists as an index and community contribution entry point.

## Available adapters

| Adapter | Output | Consumers |
|---------|--------|-----------|
| codex | `AGENTS.md` | OpenAI Codex CLI, Cursor Chat, any tool reading AGENTS.md |
| cursor | `.cursorrules` + `.cursor/rules/*.mdc` | Cursor IDE |
| copilot | `.github/copilot-instructions.md` | GitHub Copilot |
| cline | `.clinerules` | Cline VS Code extension |
| generic | `CONVENTIONS.md` | Aider, Continue.dev, universal |

## Contributing a new adapter

1. Create template `engine/crates/core/adapters/<name>/<output-file>.tera`
2. Add `pub struct <Name>Adapter` in `engine/crates/core/src/generate/adapters.rs`
3. Add to `all()` vec in same file
4. Add tests
5. Document here

Template context available: `project_name`, `has_ir`, `sections`, `quick_glance`, `stacks`, `key_conventions`, `reuse_summary`, `features_summary`, `quality_summary`, `contracts_summary`, `evolution_summary`, `runtime_summary`.

Uses Tera template engine (Jinja2-like syntax).
