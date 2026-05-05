<h1 align="center">
  <br>
  <a href="https://github.com/vynazevedo/first-plan">
    <img src="https://raw.githubusercontent.com/vynazevedo/first-plan/main/logo.png" alt="First Plan" width="350">
  </a>
  <br>
</h1>

<h4 align="center">Compiled context layer for <a href="https://claude.com/claude-code" target="_blank">Claude Code</a> on complex projects.</h4>

<p align="center">
  <a href="docs/i18n/README.pt-BR.md">Portugues (BR)</a>
</p>

<p align="center">
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  </a>
  <a href=".claude-plugin/plugin.json">
    <img src="https://img.shields.io/badge/version-0.4.1-green.svg" alt="Version">
  </a>
  <a href="https://github.com/vynazevedo/first-plan/actions/workflows/lint.yml">
    <img src="https://github.com/vynazevedo/first-plan/actions/workflows/lint.yml/badge.svg" alt="Lint">
  </a>
  <a href="https://github.com/vynazevedo/first-plan/actions/workflows/test.yml">
    <img src="https://github.com/vynazevedo/first-plan/actions/workflows/test.yml/badge.svg" alt="Test">
  </a>
  <a href="https://github.com/vynazevedo/first-plan/releases/latest">
    <img src="https://img.shields.io/badge/engine-rust%20native-darkred?logo=rust" alt="Rust Engine">
  </a>
  <a href="https://github.com/vynazevedo/first-plan">
    <img src="https://img.shields.io/badge/plugin-claude%20code-orange.svg" alt="Claude Code Plugin">
  </a>
  <a href="#stack-lenses">
    <img src="https://img.shields.io/badge/stacks-Go%20%7C%20TS%20%7C%20PHP%20%7C%20Py%20%7C%20Rust%20%7C%20TF-purple.svg" alt="Stacks">
  </a>
  <a href="#philosophy">
    <img src="https://img.shields.io/badge/protocol-plan--first-red.svg" alt="Plan-First">
  </a>
  <a href="#living-layer">
    <img src="https://img.shields.io/badge/layer-living-brightgreen.svg" alt="Living Layer">
  </a>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> -
  <a href="#how-it-works">How It Works</a> -
  <a href="#commands">Commands</a> -
  <a href="#stack-lenses">Stack Lenses</a> -
  <a href="#philosophy">Philosophy</a> -
  <a href="#configuration">Configuration</a> -
  <a href="#troubleshooting">Troubleshooting</a> -
  <a href="#license">License</a>
</p>

<p align="center">
  first-plan compiles complex projects into a structured context layer (<code>.first-plan/</code>) that lets Claude Code operate with absolute adherence to existing project patterns. It solves blind re-implementation, phantom features, spec drift, cross-session amnesia, and duplicated in-flight work - without inventing rules or imposing external best practices.
</p>

---

## Quick Start

Install via the Claude Code plugin marketplace:

```bash
/plugin marketplace add vynazevedo/first-plan
/plugin install first-plan
```

For local development:

```bash
/plugin marketplace add /local/path/to/first-plan
/plugin install first-plan@first-plan
```

Inside the target project, first run:

```bash
/first-plan:init
```

In ~3-8 minutes (depending on project size) it generates a complete `.first-plan/` with discovery, conventions, reuse index, spec-code reconciliation and git intelligence.

### Core capabilities

<table>
<tr>
<td><img src="https://img.shields.io/badge/-CORE-blue?style=flat-square" /></td>
<td><strong>Context Compilation</strong> - structured multi-layer IR optimized for AI consumption</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-ENGINE-orange?style=flat-square" /></td>
<td><strong>Stack Lens Engine</strong> - pluggable detection and analysis per stack (Go, TS, PHP, Python, Rust, Terraform, Mobile)</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-INDEX-green?style=flat-square" /></td>
<td><strong>Inverted Reuse Index</strong> - "I need X, use Y at Z"</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-MATRIX-purple?style=flat-square" /></td>
<td><strong>Spec-Code Reconciliation</strong> - feature x status x evidence matrix</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-GIT-black?style=flat-square" /></td>
<td><strong>Git Intelligence</strong> - heatmap, ownership, in-flight branches/PRs</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-LIVING-brightgreen?style=flat-square" /></td>
<td><strong>Living Layer</strong> - PostToolUse hook automatically marks sections as stale on edits</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-PROTOCOL-red?style=flat-square" /></td>
<td><strong>Plan-First Protocol</strong> - explicit human gate before execution</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-SCORING-yellow?style=flat-square" /></td>
<td><strong>Confidence-Graded Knowledge</strong> - threshold 0.7, low-confidence findings become open questions</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-STATE-cyan?style=flat-square" /></td>
<td><strong>Cross-Session State</strong> - today's session knows what yesterday's did</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-UNIVERSAL-lightgrey?style=flat-square" /></td>
<td><strong>Stack-Agnostic</strong> - generic fallback for any language</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-PROVENANCE-darkgreen?style=flat-square" /></td>
<td><strong>Provenance Tracking</strong> (v0.2.0) - every finding has source/SHA/TTL/decay - audit where it came from</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-COCHANGE-darkblue?style=flat-square" /></td>
<td><strong>Co-change Graph</strong> (v0.2.0) - "when X changes, Y also changes" - prevents incomplete PRs</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-VERIFY-magenta?style=flat-square" /></td>
<td><strong>Verification Loop</strong> (v0.2.0) - automatic lint/typecheck/tests post-execute + rollback safety net</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-NATIVE-darkred?style=flat-square" /></td>
<td><strong>Rust Engine</strong> (v0.3.0) - native binary first-plan-engine. Co-change graph for 50k commits in &lt;2s vs 5min via shell. Parallel hashing of 10k files. Zero Claude tokens for heavy lifting.</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-SEMANTIC-purple?style=flat-square" /></td>
<td><strong>BM25 Semantic Search</strong> (v0.4.0) - "I need email validation" finds <code>validateEmailRFC</code> even without an exact name match. Local SQLite index, &lt;10ms per query, 6 supported languages.</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-EMBEDDINGS-teal?style=flat-square" /></td>
<td><strong>ML Embeddings (Hybrid)</strong> (v0.4.1) - opt-in build with BGE-small (ONNX). Hybrid search combines BM25 + cosine similarity for true semantic matches.</td>
</tr>
</table>

---

## Native Engine (v0.3.0+)

Starting with v0.3.0, the plugin ships a **native Rust binary** (`first-plan-engine`) that performs the heavy lifting outside of Claude. Operations that took minutes via shell+tokens now run in seconds.

### Performance

| Operation | Shell + Claude | Native engine |
|-----------|----------------|---------------|
| Co-change graph (50k commits) | ~5 min | <2 s |
| Hash 10k files (xxh3) | ~30 s | <500 ms |
| Claude token cost | ~30k | ~0 |

### Engine installation

**Auto (recommended):** On the first invocation of `/first-plan:cochange` or `/first-plan:refresh`, the plugin offers an automatic download:

```
Native engine not detected. Download? (~5MB, 10-100x speedup)
A) Yes B) No C) Manual
```

**Manual:** Download from [Releases](https://github.com/vynazevedo/first-plan/releases) the binary matching your OS/arch. Extract and place in `${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine` (or anywhere in your `$PATH`).

**Supported platforms (v0.4.1):**
- Linux x86_64 (musl, fully static)
- Linux aarch64 (musl, fully static)
- Windows x86_64
- Linux x86_64 GNU **with ML build** (`-ml` suffix, ~50MB, embeddings via fastembed)

> macOS (x86_64 + aarch64) coming back in v0.5.0. macOS users can build from source via `cargo install --path engine/crates/cli` for now.

**From source:**
```bash
git clone https://github.com/vynazevedo/first-plan
cd first-plan/engine
cargo install --path crates/cli                       # default lean build
cargo install --path crates/cli --features=ml         # ML-enabled (embeddings)
```

### Graceful fallback

If the engine is unavailable (no network, restricted environment, opt-out), all operations **continue working** via markdown fallback. The engine is an optimization, not a requirement.

---

## Real-World Example

Output of `/first-plan:init` on a Bash dotfiles repo (~50 scripts):

```
Detected stacks: Bash (pure)
Reuse Index: 8 idiomatic patterns identified
Classified features: 21
  IMPLEMENTED: 17
  DRIFTED: 4   (alert!)
  PHANTOM: 1   (alarm!)
  IN_PROGRESS: 0
  SPEC_ONLY: 0

Average confidence: 0.94
Open questions: 8 (in 08-meta/questions.md)

Suggested next actions:
1. Review phantom feature: F03 (README claims "200+ aliases", actually: 54)
2. Technical drift: F07 (`air` installed twice in golang.sh)
3. Answer questions Q2-Q8 with /first-plan:ask
```

### Reuse Index examples

```bash
$ /first-plan:reuse "I need to detect the Linux distro"
```

Returns:

```yaml
distro_detection (confidence 0.99):
  idiom: |
    if [ -f /etc/os-release ]; then
      . /etc/os-release
      DISTRO_ID="${ID}"
    fi
  seen_in:
    - zsh.sh:14-23
    - neovim.sh:12-18
    - docker.sh:12-18
    - pentest.sh:13-17
  inconsistency: "neovim.sh uses 'unknown' as fallback instead of exit 1"
```

### Spec-Code Reconciliation example

```bash
$ /first-plan:check "CSV export endpoint"
```

Returns:

```
Match found: F12 - "CSV Export Endpoint"
Status: IMPLEMENTED (confidence 0.91)

Evidence:
- internal/handler/export.go:45 (full handler)
- internal/handler/export_test.go (8 test cases)

Recommendation: Feature already exists. Do not duplicate.
```

### Living Layer in action

After editing `README.md` in the project, the PostToolUse hook automatically marks:

```
.first-plan/cache/.stale:
README.md

.first-plan/08-meta/coverage.md (entry added):
- README.md (modified at 2026-05-04T22:02) - affects: 09-features
```

You don't need to do anything - the hook detected it. When you run `/first-plan:refresh`, only those sections get re-analyzed.

---

## Documentation

### Getting Started

- **[Quick Start](#quick-start)** - Installation and first init
- **[Commands](#commands)** - All available slash commands
- **[`.first-plan/` structure](#generated-structure)** - What gets generated in the target project

### Architecture

- **[How It Works](#how-it-works)** - Main components
- **[Stack Lenses](#stack-lenses)** - How each stack is analyzed
- **[Living Layer](#living-layer)** - Automatic invalidation hook
- **[Spec-Code Reconciliation](#spec-code-reconciliation)** - Feature matrix

### Best Practices

- **[Philosophy](#philosophy)** - 7 inviolable rules
- **[Plan-First Workflow](#plan-first-workflow)** - Discovery -> Plan -> Approval -> Execution -> Report
- **[Confidence Scoring](#confidence-scoring)** - When the plugin asks instead of guessing

### Configuration & Development

- **[Configuration](#configuration)** - Settings and customization
- **[Development](#development)** - Build, contribute, add a new stack lens
- **[Troubleshooting](#troubleshooting)** - Common issues

---

## How It Works

**Main components:**

1. **Stack Lens Engine** - detects manifests (`go.mod`, `package.json`, `composer.json`, etc), infers role (API/worker/lib/CLI/UI/infra) and routes to the matching `skills/lens-<stack>/SKILL.md`
2. **Discovery Subagent** (`discovery-analyst`) - read-only, runs Phase 1 in isolation, returns structured findings
3. **Pattern Archeologist** (`pattern-archeologist`) - extracts conventions with confidence scoring + concrete code examples
4. **Reconciliation Auditor** (`reconciliation-auditor`) - cross-references intent (docs, JIRA, GitHub issues via MCP) with evidence in code
5. **Git Intelligence** - inline read-only git commands for activity heatmap, ownership, in-flight (branches+PRs)
6. **Living Layer Hook** - `PostToolUse` watches edits and marks affected sections stale (does not regenerate - the user decides when to refresh)
7. **State Machine** - persisted in `.first-plan/07-state/STATE.md`, survives across sessions

---

## Commands

### Essential commands

<p>
<img src="https://img.shields.io/badge/-CORE-blue?style=flat-square" />
</p>

| Command | Purpose |
|---------|---------|
| `/first-plan:init` | Full compilation - creates `.first-plan/` |
| `/first-plan:refresh [section]` | Incremental refresh |
| `/first-plan:status [--verbose]` | Current layer state |

### Plan-First workflow

<p>
<img src="https://img.shields.io/badge/-PROTOCOL-red?style=flat-square" />
</p>

| Command | Purpose |
|---------|---------|
| `/first-plan:plan <feature>` | Generate plan (Phase 2), pause for approval |
| `/first-plan:execute [--dry-run]` | Execute approved plan (Phase 3), generate report |

### Query commands

<p>
<img src="https://img.shields.io/badge/-QUERY-purple?style=flat-square" />
</p>

| Command | Purpose |
|---------|---------|
| `/first-plan:why <symbol\|path>` | "Why does X exist?" |
| `/first-plan:reuse <intent>` | "What should I reuse for X?" |
| `/first-plan:risk <path>` | Catalogued risks |
| `/first-plan:ask` | Open questions for the human |
| `/first-plan:features [filter]` | Spec-Code Reconciliation matrix |
| `/first-plan:check <feature>` | "Does this already exist?" |
| `/first-plan:in-flight [--all\|--mine]` | Active branches/PRs |
| `/first-plan:hot [--days N]` | Most active areas |
| `/first-plan:owner <path>` | Who owns this file |
| `/first-plan:cochange <path>` | (v0.2.0) Files that change together with this one |
| `/first-plan:provenance <id>` | (v0.2.0) Provenance chain of a finding |
| `/first-plan:rollback [--snapshot]` | (v0.2.0) Revert last execute |

---

## Generated Structure

```
.first-plan/
├── INDEX.md                       entry point - Claude reads first
├── 00-mission/                    inferred purpose + stakeholders
├── 01-topology/                   stacks + architecture + boundaries
│   ├── stacks.md
│   ├── architecture.md
│   ├── boundaries.md
│   ├── deployments.md
│   ├── activity.md                heatmap (git)
│   └── ownership.md               per path (git)
├── 02-conventions/                extracted conventions with real examples
│   ├── naming.md
│   ├── errors.md
│   ├── testing.md
│   ├── logging.md
│   ├── di.md
│   └── security.md
├── 03-reuse/                      Inverted Reuse Index
│   ├── INDEX.md
│   ├── components.md
│   ├── utils.md
│   ├── types.md
│   ├── hooks.md
│   └── search.json                machine-readable lookup
├── 04-domain/                     glossary + entities + critical flows
├── 05-risks/                      fragile + untested + magic + debt
├── 06-rationale/                  do + dont + why (inferred decisions)
├── 07-state/                      State machine + plans + reports
│   ├── STATE.md
│   ├── in-flight.md
│   ├── sessions/                  ephemeral (gitignored)
│   ├── plans/                     active plans (Phase 2)
│   └── reports/                   execution reports (Phase 5)
├── 08-meta/                       coverage + confidence + questions + cache
└── 09-features/                   Spec-Code Reconciliation matrix
```

The plugin also appends to the target project's `.gitignore`:

```
.first-plan/cache/
.first-plan/07-state/sessions/
```

---

## Stack Lenses

V1 ships dedicated lenses for:

<p>
<img src="https://img.shields.io/badge/Go-supported-00ADD8?style=flat-square" alt="Go">
<img src="https://img.shields.io/badge/TypeScript-supported-3178C6?style=flat-square" alt="TypeScript">
<img src="https://img.shields.io/badge/PHP-supported-777BB4?style=flat-square" alt="PHP">
<img src="https://img.shields.io/badge/Python-supported-3776AB?style=flat-square" alt="Python">
<img src="https://img.shields.io/badge/Rust-supported-CE422B?style=flat-square" alt="Rust">
<img src="https://img.shields.io/badge/Terraform-supported-7B42BC?style=flat-square" alt="Terraform">
<img src="https://img.shields.io/badge/Mobile-supported-A4C639?style=flat-square" alt="Mobile">
<img src="https://img.shields.io/badge/Generic-fallback-grey?style=flat-square" alt="Generic">
</p>

| Stack | Lens | Detects |
|-------|------|---------|
| Go | `lens-go` | cmd/internal/pkg, error wrapping, context.Context, concurrency, code generation |
| TypeScript/Node | `lens-typescript` | Next.js, NestJS, Vite, Express, Astro, Remix, monorepos pnpm/turbo/nx |
| PHP | `lens-php` | Laravel, Symfony, Slim, Hyperf, PSR compliance |
| Python | `lens-python` | FastAPI, Django, Flask, Litestar, Celery, src/flat packaging |
| Rust | `lens-rust` | axum, actix-web, tokio, error handling with thiserror/anyhow |
| Terraform | `lens-terraform` | modules, state backend, environments, providers, naming/tagging |
| Mobile | `lens-mobile` | RN, Flutter, iOS Swift, Android Kotlin |
| Other | `lens-generic` | Heuristic fallback (Elixir, OCaml, Haskell, Zig, etc) |

### Add support for a new stack

Create `skills/lens-<stack>/SKILL.md` following the common contract in `skills/lens-engine/SKILL.md`. No other change is required - the engine discovers it via filesystem.

---

## Spec-Code Reconciliation

Continuous matrix between intent artifacts (docs, specs, JIRA, GitHub issues, README sections) and implementation (code, tests, PRs).

### Possible statuses

<p>
<img src="https://img.shields.io/badge/NOT__STARTED-lightgrey?style=flat-square" alt="NOT_STARTED">
<img src="https://img.shields.io/badge/SPEC__ONLY-blue?style=flat-square" alt="SPEC_ONLY">
<img src="https://img.shields.io/badge/IN__PROGRESS-yellow?style=flat-square" alt="IN_PROGRESS">
<img src="https://img.shields.io/badge/IMPLEMENTED-brightgreen?style=flat-square" alt="IMPLEMENTED">
<img src="https://img.shields.io/badge/DRIFTED-orange?style=flat-square" alt="DRIFTED">
<img src="https://img.shields.io/badge/ABANDONED-red?style=flat-square" alt="ABANDONED">
</p>

| Status | Meaning |
|--------|---------|
| `NOT_STARTED` | Intent exists but no related code |
| `SPEC_ONLY` | Documentation complete, zero implementation |
| `IN_PROGRESS` | Partial implementation, active branch, or visible TODOs |
| `IMPLEMENTED` | Code complete, with tests |
| `DRIFTED` | Code exists but diverged from spec |
| `ABANDONED` | Stale branch + partial implementation |

### Phantom Features

<p>
<img src="https://img.shields.io/badge/alert-phantom%20features-red?style=flat-square" alt="Phantom Alert">
</p>

Features marked IMPLEMENTED in code but still showing as Open in the issue tracker - high chance of imminent duplicated work. Detected and surfaced in `.first-plan/09-features/INDEX.md`.

### Sources consulted

<p>
<img src="https://img.shields.io/badge/local-docs%2Fspecs-blue?style=flat-square" alt="Local docs">
<img src="https://img.shields.io/badge/MCP-jira--mm-orange?style=flat-square" alt="JIRA MCP">
<img src="https://img.shields.io/badge/MCP-github--work-black?style=flat-square" alt="GitHub MCP">
<img src="https://img.shields.io/badge/git-history-darkgreen?style=flat-square" alt="Git history">
</p>

- Local documentation (`docs/`, `specs/`, `requirements/`, `rfcs/`, README sections)
- JIRA (via MCP `jira-mm` if available)
- GitHub Issues and PRs (via MCP `github-work` if available)
- Git history (branches, commit messages)
- Code comments (`TODO: implement`, `PLANNED:`, `FIXME`)

---

## Living Layer

<p>
<img src="https://img.shields.io/badge/hook-PostToolUse-brightgreen?style=flat-square" alt="Hook">
<img src="https://img.shields.io/badge/regenerates-no-red?style=flat-square" alt="No regen">
<img src="https://img.shields.io/badge/signals-yes-blue?style=flat-square" alt="Signal only">
</p>

The `PostToolUse` hook watches edits via Edit/Write/MultiEdit and automatically marks affected `.first-plan/` sections as stale. **It does not regenerate** - it only signals. The user decides when to run `/first-plan:refresh`.

Modified file -> affected sections mapping:

| Modified file | Sections marked stale |
|---------------|----------------------|
| Manifest (go.mod, package.json) | `01-topology/stacks` |
| `cmd/`, entry points | `01-topology/architecture` |
| Handlers / routers | `01-topology/boundaries` |
| Dockerfile, CI configs | `01-topology/deployments` |
| Source code (>= 5 files) | `01-topology/activity`, `02-conventions/*` |
| `pkg/`, `lib/`, `utils/` | `03-reuse/*` |
| Tests | `02-conventions/testing`, `05-risks/untested` |
| docs/, specs/ | `09-features/*` |

---

## Plan-First Workflow

<p>
<img src="https://img.shields.io/badge/protocol-plan--first-red?style=flat-square" alt="Plan-First">
<img src="https://img.shields.io/badge/gate-human-yellow?style=flat-square" alt="Human gate">
<img src="https://img.shields.io/badge/phases-5-blue?style=flat-square" alt="5 phases">
</p>

Mandatory protocol with explicit human gate:

```
Discovery -> Plan -> Approval -> Execution -> Report
```

### Phase 1 - Discovery

```bash
/first-plan:init
```

Result: `.first-plan/` populated. Read-only subagents run discovery in isolation and return structured findings, which are written into the target project.

### Phase 2 - Plan

```bash
/first-plan:plan <feature description>
```

Result: `.first-plan/07-state/plans/<slug>.md` containing:

1. Duplication check (queries `09-features/`)
2. Applicable reuse mapping (`03-reuse/`)
3. Files to create/modify with conceptual diff
4. Convention adherence (`02-conventions/`)
5. Risks and open questions
6. "Done" criteria + explicit out-of-scope

**Pauses for human approval.**

### Phase 3 - Approval

State: `awaiting_approval` in `STATE.md`. Nothing executes. The user approves with `/first-plan:execute` or asks for adjustments.

### Phase 4 - Execution

```bash
/first-plan:execute
```

Follows the plan precisely. Stops if any premise becomes invalid - **does not improvise**. Updates STATE every step.

### Phase 5 - Report

Generated automatically at `.first-plan/07-state/reports/<slug>.md` with:

- What was done
- What was reused vs created from scratch (with justification)
- Plan deviations (if any)
- Remaining risks
- Out-of-scope suggestions

---

## Philosophy

<p>
<img src="https://img.shields.io/badge/rules-7-red?style=flat-square" alt="7 rules">
<img src="https://img.shields.io/badge/inviolable-yes-darkred?style=flat-square" alt="Inviolable">
</p>

### 7 Inviolable Rules

1. **Reuse first** - Before creating, check `.first-plan/03-reuse/INDEX.md`. Creating from scratch requires explicit justification
2. **The project's truth lives in the project** - Do not import external best practices. If the project does it ugly but consistent, follow the ugly
3. **No new dependencies** - Use only what already exists in manifests. Adding a library requires separate approval
4. **Consistency > elegance** - Refactoring is out of scope unless requested. Suggestions go in the report's "out of scope" section
5. **Creating from scratch is the exception** - Allowed only when there is no precedent. Always justify
6. **Faithful representation** - Comments, docs, commits - all faithful to the project's writing style
7. **Strong typing** - Respect the project's type strictness. No `any`/`interface{}` if the project is strict

### Confidence Scoring

<p>
<img src="https://img.shields.io/badge/threshold-0.7-yellow?style=flat-square" alt="Threshold 0.7">
<img src="https://img.shields.io/badge/range-0.0--1.0-blue?style=flat-square" alt="Range">
</p>

Every finding has `confidence: 0.0-1.0`. Default threshold: 0.7.

| Range | Meaning |
|-------|---------|
| `>= 0.9` | high confidence, multiple converging signals |
| `0.7-0.9` | good confidence, clear signal |
| `0.5-0.7` | medium confidence, circumstantial evidence |
| `< 0.5` | low confidence, becomes a question in `08-meta/questions.md` |

**The plugin does not invent - it asks.** When confidence is low, you are consulted via `/first-plan:ask`.

---

## System Requirements

<p>
<img src="https://img.shields.io/badge/Claude%20Code-required-orange?style=flat-square" alt="Claude Code">
<img src="https://img.shields.io/badge/Git-recommended-darkgreen?style=flat-square" alt="Git">
<img src="https://img.shields.io/badge/bash-required-black?style=flat-square" alt="bash">
<img src="https://img.shields.io/badge/MCPs-optional-lightgrey?style=flat-square" alt="MCPs">
</p>

- **Claude Code**: recent version with plugin support
- **Git**: for Git Intelligence (optional - if absent, related sections stay empty with a note)
- **bash**: hooks use bash (Linux/macOS/WSL2)
- **Optional MCPs** (improve coverage):
  - `jira-mm` - reconciliation against JIRA issues
  - `github-work` - reconciliation against GitHub issues/PRs

Stack-agnostic - **does not require** specific runtimes for the analyzed stacks. The plugin reads code, it does not execute it.

---

## Configuration

The plugin requires no initial configuration - conventions are discovered during `init`. Optional customizations:

### Confidence threshold

Edit the frontmatter of `.first-plan/08-meta/confidence.md`:

```yaml
---
threshold: 0.7    # tune to 0.6 (more permissive) or 0.8 (stricter)
---
```

### Exclude paths from the hook

To prevent changes to specific paths from triggering invalidation, edit `hooks/invalidate-cache.sh` or add a rule to the project's `.gitignore`.

### Cache TTL

Git intelligence is cached for 24h by default in `08-meta/cache.json`. To force a refresh:

```bash
/first-plan:refresh --all
```

---

## Development

### Plugin structure

```
first-plan/
├── .claude-plugin/plugin.json       manifest
├── commands/                        14 slash commands
├── skills/                          17 skills (1 protocol + 1 lens-engine + 8 lenses + 7 advanced)
├── agents/                          4 subagents (discovery, reconciliation, pattern, verification)
├── hooks/                           hooks.json + invalidate-cache.sh
├── templates/                       41 templates copied to .first-plan/ on init
├── meta-templates/                  internal plugin templates (plan, report, feature)
├── engine/                          Rust workspace (core lib + cli binary)
└── README.md
```

### Add a new stack lens

1. Create `skills/lens-<stack>/SKILL.md` following the common contract in `skills/lens-engine/SKILL.md`
2. Add an entry in the detection table at `skills/lens-engine/SKILL.md`
3. No other changes required

### Test locally

```bash
/plugin marketplace add /local/path/to/first-plan
/plugin install first-plan@first-plan
cd /some/project
/first-plan:init
```

To iterate, edit plugin files and run `/plugin reload` (or restart Claude Code).

### Build the engine

```bash
cd engine
cargo build --release                           # default lean build
cargo build --release --features=ml             # ML build (with embeddings)
cargo test --workspace
cargo clippy --all-targets --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## Troubleshooting

### `.first-plan/` not created after `/first-plan:init`

- Check that the current directory is a valid project (has a manifest)
- Check that the plugin is installed: `/plugin list`
- Look at the subagent log in the init output for discovery errors

### Hook does not invalidate sections after edits

- Check permissions: `chmod +x hooks/invalidate-cache.sh`
- Check log: `tail -f ~/.first-plan-hook.log`
- Check that `.first-plan/` exists in the project directory

### Discovery very slow on a large monorepo

- Discovery applies automatic sampling for projects > 1000 files
- Use `/first-plan:refresh <section>` to refresh a single section
- Increase `time_budget_minutes` in `init` if needed

### False positives in DRIFTED

- Edit `.first-plan/09-features/<slug>.md` manually to fix the status
- The plugin respects manual edits until the next refresh of that feature

### Low average confidence

- Indicates a project with inconsistent or transitioning patterns
- Check `08-meta/questions.md` - answer the questions and refresh
- Consider documenting conventions in CLAUDE.md for future discoveries

---

## Contributing

Contributions welcome. Areas of impact:

- **New stack lenses** - Elixir, OCaml, Haskell, Scala, Clojure, Zig
- **Subagent improvements** - especially reconciliation-auditor
- **Performance optimizations** for monorepos
- **More MCP integrations** (Linear, Asana, Notion)
- **Roadmap features** - tree-sitter AST, LSP integration, multi-repo, decision archeology

Workflow:

1. Fork the repository
2. Create a feature branch
3. Implement following the plugin's own conventions (see `skills/protocol/SKILL.md`)
4. Update documentation
5. Submit a Pull Request

---

## Roadmap

<p>
<img src="https://img.shields.io/badge/v0.4.1-current-brightgreen?style=flat-square" alt="v0.4.1 current">
<img src="https://img.shields.io/badge/v0.5.0-next-blue?style=flat-square" alt="v0.5.0 next">
<img src="https://img.shields.io/badge/v1.0-vision-lightgrey?style=flat-square" alt="v1.0 vision">
</p>

### Shipped

#### v0.1.0 - Initial release

- Complete Discovery Layer (00-09)
- Spec-Code Reconciliation with phantom features detection
- Living Layer via PostToolUse hook
- 14 slash commands, 8 stack lenses, 3 read-only subagents

#### v0.2.0 - Cognitive Compiler Phase A+B

- **Provenance & Freshness Tracking** - source/SHA/TTL/decay schema
- **Co-change Graph** - change dependency from git history
- **Verification Loop** - lint/typecheck/tests post-execute
- **Rollback / Time Travel** - pre-execute snapshots

#### v0.3.0 - Native Rust Engine

- `first-plan-engine` binary in a Rust workspace
- `cochange` and `hash` subcommands (10-100x speedup)
- Cross-platform pre-built binaries (linux x86_64+arm64, windows)
- GitHub Actions CI/CD (lint, test, release)

#### v0.4.0 - BM25 Semantic Search

- Engine `index` + `search` subcommands
- Identifier-aware tokenization (snake_case + camelCase + UPPER_CASE)
- BM25 ranking over symbols extracted from Go/Rust/TS/Python/PHP
- `semantic-reuse` skill with graceful fallback
- <10ms latency, zero Claude tokens

#### v0.4.1 - ML Embeddings (current)

- Opt-in `--features=ml` feature flag
- `core::embeddings` with FastEmbedProvider (BGE-small, ONNX)
- Hybrid search combining BM25 + cosine similarity
- CLI `--mode bm25|embed|hybrid` + `--alpha` tuning
- Auto-download of models in `~/.cache/first-plan/models/`

### Planned

#### v0.5.0 - Tree-sitter AST + Bash support + Wikilinks

- Replace regex extraction with tree-sitter (exact parsing)
- Support for Bash, Ruby, Java, Kotlin, Swift, Elixir
- Engine `ast` subcommand (signature/scope/refs)
- Cross-platform ML build (linux musl + macOS + windows)
- **Obsidian-compatible `[[wikilinks]]`** between `.first-plan/` sections
  - Inspired by OpenKB - turns the layer into a navigable graph

#### v0.6.0 - LSP Integration + Watch mode

- Engine talks to gopls/pyright/typescript-language-server/rust-analyzer
- Real types via `textDocument/references` (vs heuristic)
- Replaces grep with symbol-level navigation
- **Engine `watch` subcommand** - continuous incremental refresh
  - Inspired by OpenKB - filesystem monitoring with debounced auto-refresh
  - Goes beyond the current PostToolUse hook (which only signals)

#### v0.7.0 - Multi-Repo Awareness + Multi-format docs

- `cross-repo-mapping` skill
- Detect cross-repo calls (OpenAPI, gRPC, schemas)
- `/first-plan:blast-radius <symbol>` cross-service command
- `~/.first-plan/repos.yaml` config registry of sister repos
- **Multi-format document ingestion** in reconciliation
  - Inspired by OpenKB - markitdown integration to read PDF/Word/PowerPoint specs
  - Critical for corporate environments where specs aren't markdown

#### v0.8.0 - CI/CD State + Contradiction detection

- Reads `.github/workflows/`, `.gitlab-ci.yml` (knows which checks run)
- Detects flaky tests from history
- Tags/releases comparison: "merged but not shipped"
- **Cross-section contradiction detection**
  - Inspired by OpenKB - automatic flagging of conflicting findings
  - Example: `02-conventions/naming.md` says snake_case but `06-rationale/dont.md` lists snake_case as anti-pattern
  - New `/first-plan:contradictions` command

### Long-term Vision (v1.0)

Complete Cognitive Infrastructure:

- **Bug Recurrence DB** - "this bug appeared before in #234, fixed in abc123"
- **Decision Archeology** - extracts why/because from commits/PRs/comments
- **Migration Tracker** - "47% migrated from logrus → slog"
- **Doc-Code Sync auditor**
- **Test-Code Drift detector**
- **Investigation Mode** - bug-hunt subagent
- **Onboarding Path Generator** (per role)
- **Team Awareness** (Slack/Linear sync)
- **Schema-Aware Operations** (OpenAPI/GraphQL/Protobuf breaking change detection)
- **Multi-Tool AI Sync** (Cursor + Cody + Copilot consume `.first-plan/`)

---

## License

<p>
<img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License MIT">
</p>

MIT License - see [LICENSE](./LICENSE) for full details.

Copyright (c) 2026 Vinicius Azevedo

---

## Support

- **Issues**: [GitHub Issues](https://github.com/vynazevedo/first-plan/issues)
- **Repository**: [github.com/vynazevedo/first-plan](https://github.com/vynazevedo/first-plan)
- **Author**: Vinicius Azevedo ([@vynazevedo](https://github.com/vynazevedo))

---

<p align="center">
  <img src="https://img.shields.io/badge/Built%20for-Claude%20Code-orange?style=flat-square" alt="Built for Claude Code">
  <img src="https://img.shields.io/badge/Stack-Agnostic-lightgrey?style=flat-square" alt="Stack-Agnostic">
  <img src="https://img.shields.io/badge/Protocol-Plan--First-red?style=flat-square" alt="Plan-First">
  <img src="https://img.shields.io/badge/Layer-Living-brightgreen?style=flat-square" alt="Living Layer">
</p>
