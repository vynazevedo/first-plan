# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-05-05

### Added

- **Native Rust engine** (`first-plan-engine`) for performance-critical operations
  - `cochange` subcommand: build co-change graph from git history (10-100x faster than shell loop)
  - `hash` subcommand: parallel xxh3 file hashing for cache invalidation
- Cargo workspace at `engine/` with `core` and `cli` crates
- Cross-platform pre-built binaries via GitHub Releases
  - linux x86_64 + aarch64 (musl, fully static)
  - windows x86_64
  - macOS (x86_64 + aarch64) pendente para v0.4.0 (runner queue ruim no GitHub
    Actions; usuarios macOS buildam from source via cargo install)
- GitHub Actions CI/CD pipelines
  - `lint.yml`: cargo fmt, clippy, JSON validation, shellcheck on hooks
  - `test.yml`: cargo test on linux/macos/windows
  - `release.yml`: cross-compile matrix + release publishing on tags
- New skill `engine-bootstrap` - detects/installs the native binary on first use
- `.first-plan.toml.example` configuration schema

### Changed

- Skill `co-change-analysis`: prefers engine when available, falls back to markdown
- Skill `git-intelligence`: uses engine for parallel hashing when available
- README updated with engine section, badges, and v0.3.0 capabilities

### Performance

- Co-change graph in 50k-commit monorepo: <2s (was ~5min via shell + Claude tokens)
- File hashing of 10k files: <500ms parallel (was 30s+ sequential)
- Token cost for these operations: near-zero (engine returns JSON, Claude only renders)

## [0.2.0] - 2026-05-05

### Added

- Provenance & Freshness Tracking (Pilar 6)
  - Schema with `finding_id`, `source` (file:line@SHA), `extracted_at`, `ttl`, `lifecycle`
  - Confidence decay over time (linear curve: 100% < 7d, 95% 7-30d, 85% 30-90d, ...)
  - New skill `provenance-tracker` and command `/first-plan:provenance <id>`
- Co-change Graph (Pilar 2)
  - New skill `co-change-analysis` with Union-Find clustering
  - New command `/first-plan:cochange <path>`
  - Integration in `/first-plan:plan`: alerts on missing co-changers
- Verification Loop (Pilar 1)
  - New subagent `verification-runner` (lint/typecheck/tests post-execute)
  - Diff vs plan validation
- Rollback / Time Travel (Pilar 7)
  - Auto snapshots before `/first-plan:execute`
  - New command `/first-plan:rollback`

### Changed

- Subagents (discovery, pattern, reconciliation) emit findings with provenance schema
- `/first-plan:plan` integrates co-change check
- `/first-plan:execute` creates snapshot pre-execute, invokes verification post-execute

## [0.1.1] - 2026-05-05

### Fixed

- `plan.md.template`, `report.md.template`, and `09-features/feature-template.md`
  were being copied into the target project's `.first-plan/` during `init`.
  These are internal plugin templates and should stay in the plugin only.
  Moved to `meta-templates/` and updated all references.

## [0.1.0] - 2026-05-05

### Added

- Initial release
- 14 slash commands (init, plan, execute, status, refresh, why, reuse, risk,
  ask, features, check, in-flight, hot, owner)
- 15 skills: protocol, lens-engine, 8 stack lenses (Go/TS/PHP/Python/Rust/
  Terraform/Mobile/Generic), pattern-extraction, reuse-indexing,
  reconciliation, git-intelligence, plan-emission
- 3 read-only subagents: discovery-analyst, pattern-archeologist,
  reconciliation-auditor
- 41 templates for the `.first-plan/` structure
- PostToolUse hook for Living Layer (marks sections stale on edits)

[Unreleased]: https://github.com/vynazevedo/first-plan/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/vynazevedo/first-plan/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/vynazevedo/first-plan/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/vynazevedo/first-plan/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/vynazevedo/first-plan/releases/tag/v0.1.0
