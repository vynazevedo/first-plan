# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2026-05-19

### Added

- **LSP Integration polyglot** - resolução semântica de símbolos via Language Server Protocol
  - Novo módulo `core::lsp` com 5 sub-módulos: registry, client, ops, fallback, daemon
  - Novo subcommand `first-plan-engine lsp <op>` com 7 operações:
    - `refs` - find references via `textDocument/references`
    - `def` - resolver definição via `textDocument/definition`
    - `symbols` - listar símbolos de arquivo via `textDocument/documentSymbol`
    - `hover` - tipo + docstring via `textDocument/hover`
    - `wsymbols` - busca workspace via `workspace/symbol`
    - `status` - inspecionar servers instalados + sugestões de install
    - `daemon` - gerenciar daemon de warm-servers (futuro)
- **8 LSP servers suportados** (auto-detect via manifest):
  - rust-analyzer (Rust, `Cargo.toml`)
  - gopls (Go, `go.mod`)
  - pyright (Python, `pyproject.toml`/`setup.py`/`requirements.txt`)
  - typescript-language-server (TS/JS, `package.json`/`tsconfig.json`)
  - intelephense (PHP, `composer.json`)
  - clangd (C/C++, `CMakeLists.txt`/`compile_commands.json`)
  - ruby-lsp (Ruby, `Gemfile`/`*.gemspec`)
  - lua-language-server (Lua, `.luarc.json`)
- **Detecção automática de stacks** reusa Discovery existente (manifest-based)
- **Sugestão platform-aware de install commands** (Linux/macOS/Windows)
- **Fallback graceful em 2 níveis**: LSP -> tree-sitter (quando feature ast ativa) -> grep + word-boundary
  - Plugin **funciona 100% sem nenhum LSP server instalado**
  - Output JSON marca `used_fallback: true` quando LSP indisponível
- **JSON-RPC 2.0 client** sobre stdio com Content-Length framing
  - Initialize handshake completo (capabilities, workspaceFolders)
  - Request/response correlation por ID
  - Shutdown gracioso (`shutdown` + `exit` + SIGKILL)
  - Timeouts: 30s por request, 60s no initialize
- Novo slash command `/first-plan:lsp-status` reporta cobertura LSP do projeto
- Nova skill `lsp-aware` documenta quando usar LSP vs grep
- Nova skill `lsp-bootstrap` detecta stacks faltantes e sugere instalação (nunca instala automaticamente)
- Subagents atualizados para preferir LSP quando disponível:
  - `discovery-analyst` - usa `lsp status` para mapear ambiente
  - `pattern-archeologist` - usa `lsp symbols`/`refs` para precisão semântica
  - `reconciliation-auditor` - usa `lsp wsymbols` para validar phantom features

### Changed

- Workspace bumped to 0.6.0
- Engine deps: `tokio` (rt-multi-thread + io-util + process + net + fs), `lsp-types` 0.97, `url` 2, `which` 7
- CI matrix: novo job `lsp-integration` instala rust-analyzer + gopls + typescript-language-server e roda smoke tests

### Performance

- Binary lean: 5.2 MB (+1 MB vs v0.5.3, JSON-RPC client adicionado)
- Cold start típico por server (primeira invocação):
  - rust-analyzer: 5-15s em projeto médio
  - gopls: 3-8s
  - pyright: 2-5s
  - typescript-language-server: 2-5s
- Fallback grep: <100ms tipicamente (sem dependência de server)

### Architecture

- LSP server é **processo externo**, não embarcado no binário (mantém engine lean)
- Detecção via PATH check + version probe
- Filosofia "sugere, nunca instala" - usuário sempre executa comandos próprios
- Daemon mode infraestrutura presente (stop/status); spawn warm pool fica para v0.6.1

### Limitations

- Cold start de 3-15s por server em primeira invocação (mitigado em v0.6.1 com daemon mode completo)
- Daemon spawn pool não implementado nesta versão (placeholder para v0.6.1)
- Versões de servers não são validadas contra mínimos requeridos
- Em CI matrix, smoke test só cobre 3 servers (rust-analyzer, gopls, ts-lsp) - cobertura completa via PR

## [0.5.3] - 2026-05-19

### Added

- **Output compression nativa** - reduz tokens em comandos shell sem dependência externa
  - Novo módulo `core::compress` com filtros por tool
  - Novo subcommand `first-plan-engine compress --tool <tool>`
  - Tools suportados: git-status, git-log, git-diff, git-diff-stat, git-branch,
    find, grep, rg, ls, cargo-check, cargo-test, cargo-metadata, npm-test, go-build, go-test
  - Heurísticas específicas por tool (agrupa por dir, summarize por arquivo,
    extrai só failures de tests, etc)
- Nova skill `compression-aware` documentando uso e economia esperada
- Subagents atualizados para preferir engine compress quando disponível:
  - `discovery-analyst` (find, grep, git log)
  - `pattern-archeologist` (grep)
  - `reconciliation-auditor` (git log, grep, find)
- Skill `git-intelligence` documenta compression de git log/status/diff

### Performance medida (em projetos reais)

- `find . -type f` em projeto Rust: 1.5MB -> 1.7KB (99.9% economia)
- `grep -rn "fn "` em crates/: 21KB -> 1.3KB (94% economia)
- `git log -n 50`: 1.4KB -> 963B (30% economia)
- `cargo test` em CI: 70-95% economia (mantém só FAILED + summary)

### Architecture

- Compression aplicada via tool keys (não inspeção shell), arquitetura previsível
- Fallback graceful: tool não-listada passa output direto
- `--raw-stdin` flag permite comprimir output existente sem re-executar comando
- Exit code preservado para detecção correta de falhas
- Stderr incluído para test runners (panics, errors)

### Limitations

- Outputs curtos (<500 bytes) têm ganho marginal
- Heurísticas são linha-baseadas, não AST (suficiente para shell tools)
- 15 tools cobertos vs 100+ do rtk - escolha intencional pra manter foco no fluxo do plugin

## [0.5.2] - 2026-05-05

### Added

- **Visual polish para CLI** - output formatado quando stdout é TTY
  - Headers com bordas coloridas (Unicode box-drawing)
  - Cores ANSI por contexto (cyan, green, yellow, red, dim)
  - Score bars visuais para search results
  - Strength indicators coloridos em cochange (strong/moderate/weak)
  - Status indicators (OK, WARN, ERR, INFO)
- **Progress spinners** durante operações longas em `index`
  - Spinner durante symbol collection
  - Spinner durante embeddings generation
  - Spinner durante index write
- **TTY auto-detection** - JSON mode preservado quando piped
  - Flag `--json` força JSON mesmo em TTY (CI, scripts)
  - `--output-json <path>` continua forçando JSON
  - Sem TTY (pipe, redirect): JSON automático
- Watch mode com output formatado: timestamp colorido + delta indicator (Δ) + lang badges
- Hash command com header e stats coloridos
- Search com ranking visual: #1 Yellow, score bar, kind badge dim, doc preview

### Changed

- Todos os 5 subcommands (cochange, hash, index, search, watch) suportam pretty mode
- CLI deps: crossterm 0.28, indicatif 0.17, is-terminal 0.4
- Binary size aumentou marginalmente (~1.5MB vs 1MB lean)

### Performance

- Zero overhead em JSON mode (pretty rendering só executa em TTY)
- TTY detection via is-terminal: <1ms

## [0.5.1] - 2026-05-05

### Added

- **Watch subcommand** (`first-plan-engine watch`) - filesystem monitoring com debounced events
  - notify-rs + notify-debouncer-mini
  - Default debounce 5s (interativo); 300s recomendado para production refresh triggers
  - Filtra eventos por linguagem suportada (Go, Rust, TS, Python, PHP, Bash)
  - Honra default ignores (node_modules, vendor, target, .git, .first-plan/cache, etc)
  - Output JSON line stream em stdout (parseável por skill ou wrapper)
  - `--exec '<cmd>'` opcional dispara comando externo a cada batch
  - Paths relativos ao --repo (canonicalized)
- Novo `core::watch` módulo com helper `make_event` para testes
- Inspirado em OpenKB (continuous compilation pattern)

### Performance

- Cold start <100ms, overhead de debounce desprezível
- 3 unit tests cobrindo language filtering + path relativization + ignores

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

[Unreleased]: https://github.com/vynazevedo/first-plan/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/vynazevedo/first-plan/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/vynazevedo/first-plan/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/vynazevedo/first-plan/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/vynazevedo/first-plan/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/vynazevedo/first-plan/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/vynazevedo/first-plan/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/vynazevedo/first-plan/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/vynazevedo/first-plan/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/vynazevedo/first-plan/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/vynazevedo/first-plan/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/vynazevedo/first-plan/releases/tag/v0.1.0
