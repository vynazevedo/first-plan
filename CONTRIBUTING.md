# Contributing to first-plan

Thanks for considering a contribution. This project is solo-maintained but takes external contributions seriously. Bug reports, docs improvements, cross-tool compatibility fixes, tests, and feature layers are all welcome.

This document is short on purpose. If in doubt, open an issue first and we discuss before you invest time.

## Ways to contribute

- **Bug reports** - open an issue describing what broke, minimal reproduction, expected behavior, actual behavior, environment
- **Cross-tool compatibility** - the project aims to work across Claude Code, GitHub Copilot CLI, Cursor, Codex, Ollama, etc. If your tool of choice trips over something first-plan generates, please report and PR if possible
- **Docs** - typos, unclear explanations, missing examples, translations. README exists in EN and PT-BR under `docs/i18n/`
- **New capability layers** - if you have an idea for a new layer in the IR (see roadmap in README), open an issue to discuss architecture before coding
- **Skill or command additions** - follow the frontmatter format documented below

## Before submitting a PR

Please run these checks locally. The exact commands the CI runs are in `.github/workflows/lint.yml` and `test.yml` - replicate them precisely to avoid the CI catching what you missed.

### Rust engine

```bash
cd engine
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace --release
```

If your local Rust toolchain is older than the CI's stable, run `rustup update stable` first. New clippy versions add lints that older toolchains miss.

### Frontmatter validation

Every skill, command and agent has YAML frontmatter that must pass strict validation:

```bash
pip install pyyaml
python3 tools/validate_frontmatter.py
```

Common trap: `argument-hint: [--flag N]` without quotes is parsed as a YAML flow-sequence (array). Strict loaders like GitHub Copilot CLI >= 1.0.65 silently drop the skill. Always quote: `argument-hint: "[--flag N]"`.

### JSON files

```bash
for f in $(find . -name "*.json" -not -path "./engine/target/*" -not -path "./.git/*"); do
  python3 -c "import json,sys; json.load(open('$f'))" || echo "invalid: $f"
done
```

### Shell hooks

```bash
find hooks -name "*.sh" -exec shellcheck {} +
```

## PR format

- **Small PRs preferred** - mechanical single-purpose changes get merged fast
- **PR title** should describe the outcome, not the code change. `fix: quote argument-hint YAML values so Copilot CLI loads all skills` is better than `update 8 files`
- **PR body** should explain the why. Link the issue if there is one
- **Include verification** - how did you test that your change works
- **One logical change per PR** - split unrelated changes into separate PRs

## Skill / command / agent frontmatter format

### commands/*.md

```yaml
---
description: "One sentence describing what this command does."
argument-hint: "[--flag value]"      # optional, MUST be a string
allowed-tools: [Read, Bash, Edit]    # optional, MUST be a list of strings
---
```

### skills/*/SKILL.md

```yaml
---
name: first-plan-skill-name          # kebab-case with first-plan- prefix
description: "One sentence describing when to use this skill. Multi-paragraph OK but must be a single string, so keep it on one line or wrap in quotes."
version: "0.8.0"
---
```

### agents/*.md

```yaml
---
name: agent-name
description: "One sentence describing what the agent does."
tools: Read, Glob, Grep, Bash        # comma-separated string (Claude Code convention) or list
model: sonnet                        # optional
color: purple                        # optional
---
```

## Code style

- **No emojis or icons in docs or code** unless the user explicitly requests them
- **No em-dash or en-dash** (`-` or `-`). Use plain hyphen `-`
- **Portuguese with proper accents** when writing in Portuguese (nao "publicacao", but "publicação")
- **Commit messages in Portuguese** for commits made by the project maintainer, following conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`)
- **Commit messages in English** for external contributions are also welcome
- **No comments in Rust code** unless the WHY is genuinely non-obvious. Doc comments (`///`) and module comments (`//!`) are fine

## Filing bug reports

Include:

- What you expected
- What actually happened
- Minimal reproduction (commands to run, files needed)
- Environment: OS, Claude Code version (if applicable), first-plan version, engine version (`first-plan-engine --version`)
- Whether you tried with the latest release

Bugs about cross-tool compatibility are particularly valuable and get priority. This project intends to work with any AI coding tool, so if your tool drops something first-plan produces, that is a real bug we want to fix.

## Discussing architecture changes

Before starting work on a new capability layer or a big refactor, open an issue with the design sketch. The current architecture is documented in the README's roadmap section. Feature layers planned: Quality (v0.8, shipped), Contracts (v0.9), Evolution (v0.10), Runtime (v0.11), Cross-repo (v0.12), Framework pivot (v1.0).

## Code of conduct

Be direct, be technical, respect other contributors. No harassment, no personal attacks. Disagreements about technical direction are healthy - disagreements about people are not.

## License

By contributing, you agree that your contribution will be licensed under the MIT License, same as the rest of the project.

## Questions

Open an issue with the `question` label or ping the maintainer via GitHub. This is a small project, response time is measured in days not weeks.
