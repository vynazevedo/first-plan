# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-07-20

### Added

- **Contracts Layer** - segunda camada nova da sequência planejada de 4 (Quality v0.8, Contracts v0.9, Evolution v0.10, Runtime v0.11) que capturam realidades diferentes sobre o codebase para dar ao AI base semântica de decisão. Detecta contratos formais entre componentes (OpenAPI, Protobuf, GraphQL) e faz cross-reference com o código real para classificar cada endpoint, RPC ou operation como IMPLEMENTED com forte evidência, CANDIDATE com evidência fraca requerendo validação humana, ou PHANTOM quando spec declara mas código não implementa.
- **`first-plan-engine contracts`** novo subcomando que produz `.first-plan/12-contracts/` com quatro artefatos markdown mais um JSON estruturado: `00-openapi.md` lista endpoints OpenAPI com status de implementação, `01-protobuf.md` lista services e RPCs Protobuf com status, `02-graphql.md` lista operations GraphQL com status de resolver, `03-drift.md` traz o summary consolidado com phantoms e candidates prontos para triagem, `report.json` carrega tudo estruturado para consumo programático.
- **OpenAPI 3.x parser** (via crate `openapiv3`) que detecta e parseia `openapi.yaml`, `openapi.json`, `swagger.yaml`, `api-docs.yaml` em locais comuns como root, `docs/`, `api/`, `spec/`, `openapi/`, `docs/api/`. Extrai title, version, todos os endpoints com method + path + operationId + summary + tags. Suporta formato YAML e JSON com conversão via serde_yaml transparente.
- **Protobuf parser regex-based** (sem dependência no binário protoc que complicaria CI e ambientes air-gapped) que extrai package, service names, RPC methods com request/response types e streaming type (Unary, ClientStream, ServerStream, Bidi), message count. Suficiente para cross-referenciar identificadores com código.
- **GraphQL SDL parser** (via crate `graphql-parser`) que detecta `*.graphql` e `*.gql` em qualquer diretório do projeto, extrai operations top-level (Query, Mutation, Subscription fields) e types (Object, Interface, Union, Enum, Input, Scalar). Cada operation top-level vira uma entity que crossref busca no código como resolver esperado.
- **Cross-reference engine** que pega cada endpoint OpenAPI (operationId + path), cada RPC Protobuf (serviceName.rpcName), cada operation GraphQL (name) e faz busca multi-pattern no código-fonte com word-boundary matching e filtro para paths genéricos (`/health`, `/status`). Classificação: 3+ ocorrências como IMPLEMENTED, 1-2 como CANDIDATE, zero como PHANTOM. Multi-language aware, reconhece patterns em Rust, Go, Python, TypeScript, JavaScript, Java, Kotlin, Ruby, PHP, C#.
- **Nova skill `contracts-aware`** documentando como AI deve consumir contracts IR em Plan-First: antes de implementar endpoint checar se está na spec, antes de mudar assinatura checar consumers, antes de remover código verificar se implementa contrato, detectar phantoms antes de compromissos públicos. Semântica dos três status documentada com ação recomendada para cada.

### Changed

- Workspace bumped to 0.9.0
- Engine deps: `openapiv3` 2.x e `graphql-parser` 0.4 adicionados
- 15 unit tests novos cobrindo os 4 submódulos (openapi, protobuf, graphql, crossref)

### Performance

- Contracts scan em first-plan próprio: 329ms (sem specs para parsear)
- Contracts scan em projeto sintético com OpenAPI 5-endpoints + código: 4ms
- Detecta phantoms corretamente em teste sintético: `notImplementedYet` declarado mas sem função no código foi classificado como PHANTOM, `listUsers` e `createUser` com função dedicada como IMPLEMENTED, `deleteUser` com apenas menção em roteamento como CANDIDATE

### Architecture

- Módulo `core::contracts` com 4 submódulos isolados (openapi, protobuf, graphql, crossref) cada um com testes próprios
- Padrão de design idêntico ao Quality Layer: análise local, sem dependência externa cara, output em múltiplos markdown densos + JSON, fallback graceful quando specs ausentes
- Cross-referencer usa word-boundary matching evitando falsos positivos de identifiers similares (`listUsers` vs `listUsersFast`)
- Preparação para Evolution Layer (v0.10) que precisará dessa base para validar migrations de spec

### Limitations

- Cross-repo: contratos em repo separado do handler não são detectados. Multi-repo awareness planejada para v0.12.
- Naming diferente: se handler chama `getUserById` mas operationId é `getUser`, vira Candidate. Convenção do time ajuda.
- Type-level drift: crossref detecta existência, não valida se request/response types batem. Ferramentas dedicadas como spectral ou buf complementam.
- Deprecated endpoints: não detectados nesta versão. Evolution Layer v0.10 cobrirá.

## [0.8.1] - 2026-07-20

### Added

- **Validador automático de YAML frontmatter no CI** (`tools/validate_frontmatter.py` + novo job `frontmatter` em `.github/workflows/lint.yml`) que parseia frontmatter de todos os skills, agents e commands e valida tipos e campos obrigatórios. Motivado pelo bug reportado por @thejesh23 na issue #1 onde `argument-hint: [foo]` sem aspas era interpretado como flow-sequence YAML e silenciosamente descartado por loaders estritos como GitHub Copilot CLI >= 1.0.65. Validador roda 46 arquivos em milissegundos, previne regressão desta classe de bug para sempre.
- **CONTRIBUTORS.md** listando primeira contribuição externa ao projeto por @thejesh23. Versão PT-BR em `docs/i18n/CONTRIBUTORS.pt-BR.md`.
- **CONTRIBUTING.md** com guidelines de contribuição, ritual de checks locais espelhando o CI, formato correto de frontmatter para skills, commands e agents com o bug do thejesh23 como exemplo canônico de por que as regras importam, code style rules, template de bug report. Versão PT-BR em `docs/i18n/CONTRIBUTING.pt-BR.md`.

### Fixed

- **`commands/ask.md`** tinha o mesmo bug de `argument-hint` como array não-quotado que o PR #2 do thejesh23 corrigiu em 8 outros arquivos. Foi capturado pelo validador novo. Comando agora carrega corretamente em GitHub Copilot CLI >= 1.0.65.
- **`skills/quality-aware/SKILL.md`** tinha YAML inválido no frontmatter (colon dentro de string sem aspas). Também capturado pelo validador. Skill agora parseia sem erro.

### Contributors

- [@thejesh23](https://github.com/thejesh23) - primeira contribuição externa: fix de compatibilidade cross-tool (#2 fecha #1)

## [0.8.0] - 2026-05-28

### Added

- **Quality/Validation Layer** - nova camada do IR que captura o estado de validação automática do projeto, primeira de uma sequência planejada de 4 capacidades (Quality v0.8, Schemas v0.9, Evolution v0.10, Runtime v0.11) que tornam first-plan indispensável para trabalho sério e antecipam o futuro de agentes autônomos.
- **`first-plan-engine quality`** novo subcomando que produz `.first-plan/11-quality/` com três artefatos markdown densos mais um JSON estruturado: `00-pipeline.md` lista todos os providers de CI detectados com workflows, jobs, triggers e steps, `01-coverage.md` lista todos os arquivos do projeto ordenados por % de coverage com ranges de linhas não cobertas, `02-flaky.md` lista candidatos a flaky test com score de confidence e signals que justificam a classificação, `report.json` carrega tudo de forma estruturada para consumo programático por AI ou outros tools.
- **CI workflows parser** com suporte nativo a quatro providers que cobrem mais de 90% dos projetos no mercado: GitHub Actions parseando `.github/workflows/*.yml` extraindo jobs com runs-on, steps com summary, triggers identificados como push/PR/schedule/release/workflow_dispatch, GitLab CI parseando `.gitlab-ci.yml` com jobs e stages, CircleCI parseando `.circleci/config.yml` com docker/machine executors, Jenkinsfile com parsing limitado de pipeline declarativo extraindo stages como jobs.
- **Coverage reports parser** com cinco formatos suportados que cobrem todos os ecossistemas principais: lcov info para Node e Rust via grcov e C/C++, Cobertura XML para Python e Java e JavaScript, JaCoCo XML para Java e Kotlin, jest coverage-summary.json para Node e TypeScript, go test coverprofile para Go. Cada arquivo coverage detectado retorna porcentagem, lines covered, lines total e uncovered ranges agrupados em sequências contíguas para reduzir ruído.
- **Flaky test detector via git history mining** usando três heurísticas combinadas em um score: edits isolados de arquivo de teste sem mudança correspondente em código não-test no mesmo commit, mensagens de commit com keywords suspeitas como flaky, race, timeout, intermittent, unstable, stabilize, retry, e commits revert que tocaram o test. Detecta automaticamente test files em todos os padrões comuns como `_test.go`, `_test.py`, `.test.ts`, `.spec.ts`, `_test.rs`, `test_*.py`, `_spec.rb`, paths em `tests/`, `__tests__/`, `spec/`.
- **Nova skill `quality-aware`** documentando como AI deve consumir o quality IR em três fluxos: ao planejar mudança conferir quais checks de CI vão rodar, ao decidir refactor conferir coverage do arquivo alvo, ao tocar test files conferir se aparecem no flaky ranking. Skill inclui exemplos concretos de output de plan-first quality-informed.
- **discovery-analyst subagent atualizado** para gerar quality IR após detecção de stacks e incorporar três sinais no discovery findings: providers de CI detectados, coverage overall do projeto, count de flaky candidates.

### Changed

- Workspace bumped to 0.8.0
- Engine deps: `serde_yaml 0.9` para parsing de workflows YAML, `quick-xml 0.36` com feature serialize para parsing de Cobertura e JaCoCo XML
- 15 unit tests novos cobrindo todos os parsers (4 CI providers, 5 coverage formats, flaky scoring e detection)

### Performance

- Quality scan em first-plan próprio: **1.25 segundos** com detecção completa de CI, zero coverage report disponível, e 36 commits analisados para flaky em janela de 180 dias
- Binary impact desprezível com adição de yaml e XML parsers leves
- Sem dependência externa cara, sem cliente API, sem SaaS integration - tudo local e offline

### Architecture

- Módulo `core::quality` com 3 submódulos isolados (ci, coverage, flaky) cada um auto-contido com testes próprios
- Cada parser tem fallback graceful: ausência de provider de CI, ausência de coverage report, ausência de git history são tratados como cenários válidos com mensagens explicativas em vez de falhas
- Output em três artefatos markdown densos otimizados para consumo por AI em vez de um arquivo monolítico, facilitando inclusão seletiva no contexto
- Padrão de design replicável para próximas camadas planejadas (Schemas, Evolution, Runtime, Cross-repo)

### Validação real

- Rodando quality no próprio first-plan detectou automaticamente `engine/crates/cli/tests/cli_test.rs` como flaky com score 1.65 e signals corretos: três commits com keywords flaky/race/timeout e três edits isoladas. Esse é exatamente o teste do daemon que causou problemas em macOS runner ao longo desta semana, validando a precisão da heurística sem precisar de CI logs externos.

## [0.7.1] - 2026-05-28

### Fixed

- **Windows build/release agora compila** - daemon module estava usando
  `tokio::net::{UnixListener, UnixStream}` sem cfg-gate, quebrando build
  no target `x86_64-pc-windows-msvc` desde v0.6.1 (release workflow falhou
  silenciosamente em v0.6.1 e v0.7.0 - **nenhum binario foi publicado**
  entre v0.6.0 e v0.7.0, esta release restaura o cross-platform publish).
- **Daemon API cross-platform** - estruturas (`DaemonRequest`,
  `DaemonResponse`, `DaemonStatus`) e funcoes utilitarias (`socket_path`,
  `pid_path`, `read_pid`) agora compilam em qualquer plataforma. Logica
  Unix-only isolada em `#[cfg(unix)] mod unix_impl`, com stub
  `#[cfg(not(unix))]` retornando erro descritivo no Windows.
- **clippy 1.96 compat** - `sort_by(|a,b| b.1.cmp(&a.1))` substituido por
  `sort_by_key(|x| std::cmp::Reverse(x.1))` em `quick.rs` (novo lint
  `unnecessary_sort_by` em Rust 1.96 estava bloqueando o lint workflow).
- **Tests daemon platform-aware** - `lsp_daemon_status_when_not_running` e
  `lsp_daemon_start_then_status_then_stop` agora `#[cfg(unix)]` (Windows
  pula esses testes ja que daemon eh unix-only). Status check usa retry
  loop de 30 tentativas para evitar race condition com daemon residual de
  outros jobs CI paralelos (macOS runners flutuam mais).

### Changed

- Workspace bumped to 0.7.1
- Sem mudancas funcionais vs v0.7.0 - esta release existe apenas para
  destravar o pipeline de publicacao de binarios

### Migration

Nada a fazer. Quem usou v0.7.0 (via plugin install ou build from source
em Linux/macOS) ja tem todo o funcional. v0.7.1 apenas garante que os
binarios pre-compilados sao publicados no GitHub Releases.

## [0.7.0] - 2026-05-28

### BREAKING CHANGES

- **Plugin renamed: `first-plan` → `fp`**. Todos os slash commands mudaram de prefixo:
  - `/first-plan:init` → `/fp:init`
  - `/first-plan:check` → `/fp:check`
  - `/first-plan:cochange` → `/fp:cochange`
  - ...e os demais 16 comandos
- **Comando de install mudou**:
  - Antes: `/plugin install first-plan`
  - Depois: `/plugin install fp` (ou `/plugin install fp@first-plan` em local development)
- Marca/repo/binário **mantidos**: o repo continua `vynazevedo/first-plan`, o engine continua `first-plan-engine`, a descrição/marca segue "first-plan". Mudança é só no nome do plugin (atalho de invocação).
- **Migração**: reinstalar plugin com novo nome. Configurações em `.first-plan/` são preservadas.

### Added

- **`/fp:quick` - 5-second project glance** (novo comando flagship)
  - Gera `.first-plan/quick/00-glance.md` em 1-5 segundos
  - Stacks detectadas (root + 1 nível: detecta Cargo.toml em `engine/`, package.json em `frontend/`, etc)
  - Entry points classicos (`main.*`, `index.*`, `server.*`, `app.py`)
  - Top 25 símbolos via heurística (pub fn, struct, class, def, export function, etc)
  - Atividade git 90d: últimos 10 commits, top 5 hot files, top 5 autores
  - Naming convention detectada (snake_case vs camelCase vs kebab-case)
  - Test framework detectado (cargo test, go test, pytest, jest, vitest, etc)
  - Suggested commands extraídos de manifests + scripts package.json + targets Makefile
- **Engine subcommand `first-plan-engine quick`**
  - `--root <path>` projeto alvo
  - `--output <path>` escreve markdown renderizado pra arquivo
  - `--markdown` saída markdown em stdout
  - `--json` força JSON em TTY
  - Pretty mode com cores: stacks, symbols, recent commits, hot files, conventions
- **`core::quick::glance()`** retorna `GlanceReport` estruturado
- **`core::quick::render_markdown()`** converte report em 1 página markdown denso
- 5 unit tests cobrindo stack detection, symbol sampling, command suggestion

### Changed

- Workspace bumped to 0.7.0
- README EN + PT-BR hero pivotado: headline benefit-oriented, `/fp:quick` antes do `/fp:init`
- Roadmap badge atualizado: v0.7.0 current, v0.7.1 next

### Performance

- `/fp:quick` em first-plan próprio (projeto Rust + frontend): **1.3-1.5 segundos**
- Output: ~3.5KB markdown, ~100 linhas
- Sem indexing, sem subagents, sem LSP cold start
- Binary impact: negligível (~100KB)

### Architecture

- `/fp:quick` não duplica `/fp:init` - é uma **camada paralela** em `.first-plan/quick/`
- Roda tudo via engine + Claude lendo JSON (zero subagents = mais leve em tokens)
- Heurística-first: prioriza velocidade sobre completude (oposto do init que prioriza precisão)
- Stack detection olha root + 1 nível: cobre 90% dos projetos sem fazer walk profundo

### Migration guide (v0.6.x → v0.7.0)

```bash
# 1. Desinstalar plugin antigo
/plugin uninstall first-plan

# 2. Atualizar marketplace
/plugin marketplace update first-plan

# 3. Instalar com novo nome
/plugin install fp

# 4. (opcional) Rodar quick glance pra ver o que mudou
/fp:quick
```

Configurações em `.first-plan/` do projeto-alvo são **preservadas** - o rename é só no atalho de invocação.

## [0.6.1] - 2026-05-26

### Added

- **LSP Daemon Mode** - elimina cold start de 3-15s a partir da segunda chamada
  - Novo subcomando `first-plan-engine lsp daemon start --root <path> [--idle-minutes 30]`
  - Mantem pool de `LspClient` warm (HashMap<ServerId, Arc<LspClient>>)
  - Lazy spawn por server: primeira request por server type paga cold start, demais respondem em <100ms
  - Auto-shutdown apos `--idle-minutes` minutos sem requests (default 30)
- **IPC sobre Unix socket** com protocolo JSON line-delimited
  - Request: `{"id": N, "op": "refs|def|symbols|hover|wsymbols|status|shutdown", "args": {...}}`
  - Response: `{"id": N, "result": <Value>}` ou `{"id": N, "error": "msg"}`
  - Socket em `${XDG_RUNTIME_DIR}/first-plan-engine.sock`
  - Pid file em `${XDG_RUNTIME_DIR}/first-plan-engine.pid` (com check via `kill -0` em Unix)
- **Auto-routing transparente** - todas as ops LSP (`refs`, `def`, `symbols`, `hover`, `wsymbols`) checam `daemon::is_running()` antes de spawn fresh
  - Se daemon ativo: roteia via socket
  - Se daemon offline ou erro de IPC: fallback automatico para spawn direto
  - Skills/subagents nao precisam saber se daemon esta ativo - transparente
- **Status enriquecido** - `lsp daemon status` reporta:
  - `running`, `pid`, `uptime_seconds`, `idle_seconds`
  - `warm_servers` (lista de ServerId ja vivos no pool)
  - `socket_path`, `pid_file`
- **Stop gracioso** - `lsp daemon stop` envia request shutdown via socket
  - Daemon faz `LspClient::shutdown()` em cada client warm antes de exit
  - Fallback: SIGTERM via pid se socket nao responder em 2s

### Changed

- Workspace bumped to 0.6.1
- Engine deps: `libc 0.2` (target unix) para `kill(pid, 0)` check
- Skill `lsp-aware` atualizada com secao Daemon mode + lifecycle + recomendacoes
- Slash command `/fp:lsp-status` mostra daemon status no relatorio

### Performance

- Binary: 5.2 MB (sem mudanca significativa vs v0.6.0)
- LSP op com daemon warm: **<100ms** (vs 3-15s cold start direto)
- Overhead de IPC sobre Unix socket: ~1-2ms por request
- Daemon idle RAM (sem servers spawned): ~10 MB
- Daemon com rust-analyzer + gopls warm: ~500MB-1.5GB (depende do tamanho do projeto)

### Architecture

- Daemon usa `tokio::net::UnixListener` + `oneshot::channel` para shutdown signal
- Background task verifica idle timeout a cada 30s
- Connection handler: read 1 linha JSON, dispatch, write 1 linha JSON, close
- Single instance enforced via pid file + `kill -0` check
- Sem detach manual: usuario backgrounding com `nohup ... &` ou systemd unit

### Limitations

- Apenas Unix (Linux/macOS). Windows fica para v0.7 (named pipes)
- 1 daemon por usuario (socket path fixo). Multi-root requer kill + restart com novo root
- Sem reconnect automatico: se servidor LSP morre, daemon mantem o erro ate restart
- Sem health check periodico nos LspClients warm (futuro: ping `$/cancelRequest` ou similar)

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
- Novo slash command `/fp:lsp-status` reporta cobertura LSP do projeto
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
- `/fp:reuse` atualizado com Passo 0 (BM25 path) antes do fallback
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
  - New skill `provenance-tracker` and command `/fp:provenance <id>`
- Co-change Graph (Pilar 2)
  - New skill `co-change-analysis` with Union-Find clustering
  - New command `/fp:cochange <path>`
  - Integration in `/fp:plan`: alerts on missing co-changers
- Verification Loop (Pilar 1)
  - New subagent `verification-runner` (lint/typecheck/tests post-execute)
  - Diff vs plan validation
- Rollback / Time Travel (Pilar 7)
  - Auto snapshots before `/fp:execute`
  - New command `/fp:rollback`

### Changed

- Subagents (discovery, pattern, reconciliation) emit findings with provenance schema
- `/fp:plan` integrates co-change check
- `/fp:execute` creates snapshot pre-execute, invokes verification post-execute

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

[Unreleased]: https://github.com/vynazevedo/first-plan/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/vynazevedo/first-plan/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/vynazevedo/first-plan/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/vynazevedo/first-plan/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/vynazevedo/first-plan/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/vynazevedo/first-plan/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/vynazevedo/first-plan/compare/v0.6.0...v0.6.1
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
