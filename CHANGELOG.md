# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-05-05

### Added

- **Bash extractor** (regex-based) - dotfiles e shell scripts agora indexaveis
  - Detecta `function name() { ... }` e `name() { ... }` (POSIX-style)
  - Suporta `.sh`, `.bash`, `.zsh` extensions + dotfiles (`.bashrc`, `.zshrc`, etc)
  - 4 unit tests cobrindo edge cases
- **Tree-sitter AST extraction** (opt-in via `--features=tree-sitter`)
  - Extracao precisa via parsing real (vs regex aproximado)
  - 5 linguagens: Rust, Go, Python, TypeScript, Bash
  - Detecta methods automaticamente (function dentro de class/impl/struct)
  - Enriquece com docstrings via fallback para regex doc extractor
  - Em projeto Rust real: 184 simbolos vs 129 com regex (+43% precisao)
  - Latencia ~10ms por arquivo, ~2s pra projeto medio
- **Wikilinks Obsidian-compatible** entre arquivos do `.first-plan/`
  - Sintaxe `[[secao/arquivo]]` (sem extensao .md, sem path prefix)
  - Templates atualizados (INDEX.md tem 30+ wikilinks)
  - Skill protocol documenta a convencao
  - Habilita navegacao como graph no Obsidian/Logseq
- Novo `core::ast` modulo com tree-sitter queries por linguagem

### Changed

- `extract_symbols` agora prefere AST quando feature `tree-sitter` ativada
  - Fallback automatico para regex se AST retorna empty
  - Mantem 100% backward compat (default build = regex apenas)
- Release workflow: novo target `x86_64-unknown-linux-musl-ast` (~10MB)
  - 3 builds default (~1MB cada)
  - 1 build ml (~50MB)
  - 1 build tree-sitter (~10MB)

### Performance

- Bash extraction: 39 simbolos do dotfiles em <300ms
- Tree-sitter Rust extraction: 184 simbolos em 1854ms (vs 696ms regex, mas +43% accuracy)

## [0.4.1] - 2026-05-05

### Added

- **Embeddings ML opt-in** (Pillar 5 do v2.0 roadmap, completo)
  - Feature flag `--features=ml` em core e cli (default build continua lean ~1MB)
  - Modulo `core::embeddings` com `EmbeddingProvider` trait + `FastEmbedProvider`
    via fastembed-rs (BGE-small-en-v1.5, 384 dims, ONNX backend)
  - Cosine similarity helper + utilities para serializar f32 vectors em SQLite BLOB
  - Modelos auto-baixados em `~/.cache/first-plan/models/` (gerenciado por fastembed)
- **Hybrid search** combinando BM25 + cosine similarity
  - Funcao `search::search_hybrid(db, query, q_emb, limit, alpha)`
  - Funcao `search::search_embed(db, q_emb, limit)` para cosine puro
  - Alpha tuning: 0.3 default (favorece embeddings com fallback BM25)
  - Normalizacao linear: BM25/max_score + (cosine+1)/2, ambos em [0,1]
- CLI `search` agora aceita `--mode bm25|embed|hybrid` e `--alpha 0.3`
- CLI `index` agora aceita `--embed` para gerar embeddings ao indexar
- Schema do indice tem coluna `embedding BLOB` (NULL quando nao gerada)
- Meta-table guarda `has_embeddings` e `embedding_dim`
- Skill `semantic-reuse` atualizado com tabela de modes e capability detection

### Build

- Crate `openssl` com feature `vendored` adicionada quando `--features=ml`
  (necessaria pra fastembed/hf-hub baixar modelos via HTTPS)
- Release workflow: nova entrada `x86_64-unknown-linux-gnu-ml` na matrix
- ML builds tem sufixo `-ml` no nome do artefato

### Limitacoes conhecidas

- ML build apenas para `x86_64-unknown-linux-gnu` em v0.4.1
  (musl + ONNX + openssl-vendored e fragil; aarch64 + windows + macOS planejados v0.5.0)
- ML binario significativamente maior (~50MB vs 1MB do default)
- Cold start ~1-3s para carregar modelo BGE
- Latencia query: ~50-100ms (embedding generation) vs <10ms BM25

## [0.4.0] - 2026-05-05

### Added

- **Semantic Search via BM25** (Pillar 5 do v2.0 roadmap, parcial)
  - Engine subcommand `index`: extrai simbolos de Go/Rust/TS/JS/Python/PHP via regex,
    constroi indice BM25 em SQLite com tokenizacao identifier-aware (snake_case +
    camelCase + PascalCase + UPPER_CASE + letter/digit boundaries)
  - Engine subcommand `search`: query natural-language ranqueada por BM25 (k1=1.5, b=0.75)
  - Stop words filtradas, tokens curtos descartados (exceto digitos)
  - Storage: SQLite com indices em `tokens` e `symbols`, sqlite bundled (zero deps externas)
- Novo skill `semantic-reuse` - usa engine BM25 quando disponivel, fallback markdown
- `/first-plan:reuse` atualizado com Passo 0 (BM25 path) antes do fallback
- Schemas `first-plan-index-v1` e `first-plan-search-v1` adicionados ao output

### Performance

- 129 simbolos do source Rust indexados em 696ms
- Query "parse git log" retorna `parse_log_output` como top hit (score 10.83)
- Query "cluster detection" -> `detect_clusters` (score 7.57, matches 100%)
- Query "BM25 search index" -> funcao `search` (score 11.09)
- Latencia query: <10ms tipico

### Linguagens suportadas (extracao de simbolos)

- Go: func, type, const, var, methods (detectado por receiver)
- Rust: fn, struct, enum, trait, const, static
- TypeScript/JavaScript: function, arrow function, class, interface, type, const
- Python: def, async def, class, methods (detectado por indentacao)
- PHP: function, class, interface, methods

Linguagens nao listadas caem no fallback grep ate v0.5.0 (tree-sitter).

### Limitacoes conhecidas (v0.4.0)

- Bash/shell: nao tem extractor; dotfiles puros nao geram simbolos. Fallback grep
  continua funcionando para co-change e outras analises.
- Embeddings ML: planejado para v0.4.1 como build opt-in (`--features=ml`).
  Mantem binario lean por enquanto.

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
