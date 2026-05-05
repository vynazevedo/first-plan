<h1 align="center">
  <br>
  <a href="https://github.com/vynazevedo/first-plan">
    <img src="https://raw.githubusercontent.com/vynazevedo/first-plan/main/logo.png" alt="First Plan" width="350">
  </a>
  <br>
</h1>

<h4 align="center">Camada compilada de contexto para <a href="https://claude.com/claude-code" target="_blank">Claude Code</a> em projetos complexos.</h4>

<p align="center">
  <a href="../../README.md">English</a>
</p>

<p align="center">
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  </a>
  <a href=".claude-plugin/plugin.json">
    <img src="https://img.shields.io/badge/version-0.4.0-green.svg" alt="Version">
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
  first-plan compila projetos complexos em uma camada estruturada de contexto (<code>.first-plan/</code>), permitindo que o Claude Code opere com aderência absoluta aos padrões existentes do projeto. Resolve re-implementação cega, phantom features, drift de specs, cross-session amnesia e duplicação de trabalho em flight - sem inventar regras nem impor best practices externas.
</p>

---

## Quick Start

Instale via marketplace plugin do Claude Code:

```bash
/plugin marketplace add vynazevedo/first-plan
/plugin install first-plan
```

Para uso local em desenvolvimento:

```bash
/plugin marketplace add /caminho/local/first-plan
/plugin install first-plan@first-plan
```

No projeto-alvo, na primeira vez:

```bash
/first-plan:init
```

Em ~3-8 minutos (dependendo do tamanho do projeto), gera `.first-plan/` completo com discovery, conventions, reuse index, spec-code reconciliation e git intelligence.

### Capacidades principais

<table>
<tr>
<td><img src="https://img.shields.io/badge/-CORE-blue?style=flat-square" /></td>
<td><strong>Context Compilation</strong> - camada IR multi-layer estruturada para IA</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-ENGINE-orange?style=flat-square" /></td>
<td><strong>Stack Lens Engine</strong> - detecção e análise plugavel por stack (Go, TS, PHP, Python, Rust, Terraform, Mobile)</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-INDEX-green?style=flat-square" /></td>
<td><strong>Reuse Index invertido</strong> - "preciso de X, use Y em Z"</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-MATRIX-purple?style=flat-square" /></td>
<td><strong>Spec-Code Reconciliation</strong> - matriz feature x status x evidência</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-GIT-black?style=flat-square" /></td>
<td><strong>Git Intelligence</strong> - heatmap, ownership, in-flight branches/PRs</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-LIVING-brightgreen?style=flat-square" /></td>
<td><strong>Living Layer</strong> - hook PostToolUse marca seções stale automaticamente</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-PROTOCOL-red?style=flat-square" /></td>
<td><strong>Plan-First Protocol</strong> - gate humano explícito antes de executar</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-SCORING-yellow?style=flat-square" /></td>
<td><strong>Confidence-Graded Knowledge</strong> - threshold 0.7, dúvidas viram perguntas</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-STATE-cyan?style=flat-square" /></td>
<td><strong>Cross-Session State</strong> - sessão de hoje sabe o que sessão de ontem fez</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-UNIVERSAL-lightgrey?style=flat-square" /></td>
<td><strong>Stack-Agnostic</strong> - fallback genérico para qualquer linguagem</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-PROVENANCE-darkgreen?style=flat-square" /></td>
<td><strong>Provenance Tracking</strong> (v0.2.0) - cada finding tem source/SHA/TTL/decay - audita de onde veio</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-COCHANGE-darkblue?style=flat-square" /></td>
<td><strong>Co-change Graph</strong> (v0.2.0) - "quando X muda, Y também" - previne PRs incompletos</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-VERIFY-magenta?style=flat-square" /></td>
<td><strong>Verification Loop</strong> (v0.2.0) - lint/typecheck/tests automaticos pos-execute + rollback safety net</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-NATIVE-darkred?style=flat-square" /></td>
<td><strong>Rust Engine</strong> (v0.3.0) - binario nativo first-plan-engine. Co-change graph 50k commits em &lt;2s vs 5min via shell. Hash 10k arquivos paralelo. Zero token Claude pra heavy lifting.</td>
</tr>
<tr>
<td><img src="https://img.shields.io/badge/-SEMANTIC-purple?style=flat-square" /></td>
<td><strong>Semantic Search BM25</strong> (v0.4.0) - "preciso de validacao de email" encontra `validateEmailRFC` mesmo sem nome exato. Indice SQLite local, &lt;10ms por query, 6 linguagens suportadas.</td>
</tr>
</table>

---

## Native Engine (v0.3.0+)

A partir da v0.3.0, o plugin inclui um **binário nativo Rust** (`first-plan-engine`) que faz heavy lifting fora do Claude. Operações que levavam minutos via shell+tokens agora rodam em segundos.

### Performance

| Operação | Shell + Claude | Engine nativo |
|----------|----------------|---------------|
| Co-change graph (50k commits) | ~5 min | <2 s |
| Hash 10k arquivos (xxh3) | ~30 s | <500 ms |
| Custo em tokens Claude | ~30k | ~0 |

### Instalação do engine

**Auto (recomendado):** Na primeira invocação de `/first-plan:cochange` ou `/first-plan:refresh`, o plugin oferece download automatico:

```
Engine nativo nao detectado. Baixar? (~5MB, melhora 10-100x)
A) Sim B) Nao C) Manual
```

**Manual:** Download em [Releases](https://github.com/vynazevedo/first-plan/releases) o binário matching seu OS/arch. Extraia e coloque em `${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine` (ou no `$PATH`).

**Plataformas suportadas (v0.3.0):**
- Linux x86_64 (musl, fully static)
- Linux aarch64 (musl, fully static)
- Windows x86_64

> macOS (x86_64 + aarch64) sera reativado em v0.4.0. Por enquanto, usuarios macOS podem buildar from source via `cargo install --path engine/crates/cli`.

**From source:**
```bash
git clone https://github.com/vynazevedo/first-plan
cd first-plan/engine
cargo install --path crates/cli
```

### Graceful fallback

Se o engine não estiver disponível (sem rede, ambiente restrito, opt-out), todas as operações **continuam funcionando** via fallback markdown. Engine é optimization, não requirement.

---

## Real-World Example

Output do `/first-plan:init` rodado num repositorio Bash de dotfiles (~50 scripts):

```
Stacks detectadas: Bash (puro)
Reuse Index: 8 padroes idiomaticos identificados
Features classificadas: 21
  IMPLEMENTED: 17
  DRIFTED: 4   (alerta!)
  PHANTOM: 1   (alarm!)
  IN_PROGRESS: 0
  SPEC_ONLY: 0

Confidence media: 0.94
Perguntas abertas: 8 (em 08-meta/questions.md)

Proximas acoes sugeridas:
1. Revisar phantom feature: F03 (README claim "200+ aliases", real: 54)
2. Drift tecnico: F07 (`air` instalado 2x em golang.sh)
3. Responder questions Q2-Q8 com /first-plan:ask
```

### Exemplos do Reuse Index gerado

```bash
$ /first-plan:reuse "preciso detectar a distro Linux"
```

Retorna:

```yaml
distro_detection (confidence 0.99):
  idiom: |
    if [ -f /etc/os-release ]; then
      . /etc/os-release
      DISTRO_ID="${ID}"
    fi
  visto_em:
    - zsh.sh:14-23
    - neovim.sh:12-18
    - docker.sh:12-18
    - pentest.sh:13-17
  inconsistencia: "neovim.sh usa 'unknown' como fallback ao inves de exit 1"
```

### Exemplo de Spec-Code Reconciliation

```bash
$ /first-plan:check "endpoint de export CSV"
```

Retorna:

```
Match encontrado: F12 - "CSV Export Endpoint"
Status: IMPLEMENTED (confidence 0.91)

Evidencia:
- internal/handler/export.go:45 (handler completo)
- internal/handler/export_test.go (8 test cases)

Recomendacao: Feature ja existe. Nao duplicar.
```

### Living Layer em acao

Apos editar `README.md` no projeto, o hook PostToolUse marca automaticamente:

```
.first-plan/cache/.stale:
README.md

.first-plan/08-meta/coverage.md (entry adicionado):
- README.md (modificado em 2026-05-04T22:02) - afeta: 09-features
```

Voce nao precisa fazer nada - o hook detectou. Quando rodar `/first-plan:refresh`, so essas secoes sao re-analisadas.

---

## Documentation

### Getting Started

- **[Quick Start](#quick-start)** - Instalação e primeiro init
- **[Commands](#commands)** - Todos os slash commands disponíveis
- **[Estrutura `.first-plan/`](#estrutura-criada-no-projeto-alvo)** - O que é gerado no projeto-alvo

### Architecture

- **[How It Works](#how-it-works)** - Componentes principais
- **[Stack Lenses](#stack-lenses)** - Como cada stack é analisada
- **[Living Layer](#living-layer)** - Hook de invalidação automática
- **[Spec-Code Reconciliation](#spec-code-reconciliation)** - Matriz de features

### Best Practices

- **[Philosophy](#philosophy)** - 7 regras invioláveis
- **[Plan-First Workflow](#plan-first-workflow)** - Discovery -> Plan -> Approval -> Execution -> Report
- **[Confidence Scoring](#confidence-scoring)** - Quando o plugin pergunta em vez de inventar

### Configuration & Development

- **[Configuration](#configuration)** - Settings e customização
- **[Development](#development)** - Build, contribute, adicionar nova stack lens
- **[Troubleshooting](#troubleshooting)** - Problemas comuns

---

## How It Works

**Componentes principais:**

1. **Stack Lens Engine** - detecta manifestos (`go.mod`, `package.json`, `composer.json`, etc), infere papel (API/worker/lib/CLI/UI/infra) e roteia para `skills/lens-<stack>/SKILL.md` correspondente
2. **Discovery Subagent** (`discovery-analyst`) - read-only, executa Fase 1 inteira em contexto isolado, retorna findings estruturados
3. **Pattern Archeologist** (`pattern-archeologist`) - extrai convenções com confidence scoring + exemplos concretos do código
4. **Reconciliation Auditor** (`reconciliation-auditor`) - cruza intenções (docs, JIRA, GitHub issues via MCP) com evidência no código
5. **Git Intelligence** - inline, comandos git read-only para activity heatmap, ownership, in-flight (branches+PRs)
6. **Living Layer Hook** - `PostToolUse` monitora edits e marca seções afetadas como stale (não regenera - usuário decide quando refreshar)
7. **State Machine** - persistência em `.first-plan/07-state/STATE.md` atravessa sessões

---

## Commands

### Comandos essenciais

<p>
<img src="https://img.shields.io/badge/-CORE-blue?style=flat-square" />
</p>

| Comando | Função |
|---------|--------|
| `/first-plan:init` | Compilação completa - cria `.first-plan/` |
| `/first-plan:refresh [section]` | Atualização incremental |
| `/first-plan:status [--verbose]` | Estado atual da camada |

### Workflow Plan-First

<p>
<img src="https://img.shields.io/badge/-PROTOCOL-red?style=flat-square" />
</p>

| Comando | Função |
|---------|--------|
| `/first-plan:plan <feature>` | Gera plano (Fase 2), pausa para aprovação |
| `/first-plan:execute [--dry-run]` | Executa plano aprovado (Fase 3), gera report |

### Comandos de query

<p>
<img src="https://img.shields.io/badge/-QUERY-purple?style=flat-square" />
</p>

| Comando | Função |
|---------|--------|
| `/first-plan:why <símbolo\|path>` | "Por que X existe?" |
| `/first-plan:reuse <intenção>` | "O que reusar pra X?" |
| `/first-plan:risk <path>` | Riscos catalogados |
| `/first-plan:ask` | Perguntas abertas para o humano |
| `/first-plan:features [filter]` | Matriz Spec-Code Reconciliation |
| `/first-plan:check <feature>` | "Isto já existe?" |
| `/first-plan:in-flight [--all\|--mine]` | Branches/PRs ativos |
| `/first-plan:hot [--days N]` | Áreas mais ativas |
| `/first-plan:owner <path>` | Quem domina esse arquivo |
| `/first-plan:cochange <path>` | (v0.2.0) Arquivos que mudam junto com este |
| `/first-plan:provenance <id>` | (v0.2.0) Cadeia de proveniência de um finding |
| `/first-plan:rollback [--snapshot]` | (v0.2.0) Reverte último execute |

---

## Estrutura criada no projeto-alvo

```
.first-plan/
├── INDEX.md                       entry point - Claude le primeiro
├── 00-mission/                    propósito + stakeholders inferidos
├── 01-topology/                   stacks + arquitetura + boundaries
│   ├── stacks.md
│   ├── architecture.md
│   ├── boundaries.md
│   ├── deployments.md
│   ├── activity.md                heatmap (git)
│   └── ownership.md               por path (git)
├── 02-conventions/                convenções extraídas com exemplos reais
│   ├── naming.md
│   ├── errors.md
│   ├── testing.md
│   ├── logging.md
│   ├── di.md
│   └── security.md
├── 03-reuse/                      Reuse Index invertido
│   ├── INDEX.md
│   ├── components.md
│   ├── utils.md
│   ├── types.md
│   ├── hooks.md
│   └── search.json                machine-readable lookup
├── 04-domain/                     glossário + entidades + flows críticos
├── 05-risks/                      fragile + untested + magic + debt
├── 06-rationale/                  do + dont + why (decisões inferidas)
├── 07-state/                      State machine + planos + reports
│   ├── STATE.md
│   ├── in-flight.md
│   ├── sessions/                  efêmero (gitignored)
│   ├── plans/                     planos ativos (Fase 2)
│   └── reports/                   reports de execução (Fase 5)
├── 08-meta/                       coverage + confidence + questions + cache
└── 09-features/                   Spec-Code Reconciliation matrix
```

E modifica `.gitignore` do projeto-alvo adicionando:

```
.first-plan/cache/
.first-plan/07-state/sessions/
```

---

## Stack Lenses

V1 com lenses dedicadas:

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

| Stack | Lens | Detecta |
|-------|------|---------|
| Go | `lens-go` | cmd/internal/pkg, error wrapping, context.Context, concurrency, code generation |
| TypeScript/Node | `lens-typescript` | Next.js, NestJS, Vite, Express, Astro, Remix, monorepos pnpm/turbo/nx |
| PHP | `lens-php` | Laravel, Symfony, Slim, Hyperf, PSR compliance |
| Python | `lens-python` | FastAPI, Django, Flask, Litestar, Celery, packaging src/flat |
| Rust | `lens-rust` | axum, actix-web, tokio, error handling com thiserror/anyhow |
| Terraform | `lens-terraform` | módulos, state backend, environments, providers, naming/tagging |
| Mobile | `lens-mobile` | RN, Flutter, iOS Swift, Android Kotlin |
| Outras | `lens-generic` | Fallback heurístico (Elixir, OCaml, Haskell, Zig, etc) |

### Adicionar suporte a nova stack

Crie `skills/lens-<stack>/SKILL.md` seguindo o contrato comum em `skills/lens-engine/SKILL.md`. Não há outras mudanças necessárias - o engine descobre via filesystem.

---

## Spec-Code Reconciliation

Matriz contínua entre artefatos de intenção (docs, specs, JIRA, GitHub issues, README sections) e implementação (código, testes, PRs).

### Statuses possíveis

<p>
<img src="https://img.shields.io/badge/NOT__STARTED-lightgrey?style=flat-square" alt="NOT_STARTED">
<img src="https://img.shields.io/badge/SPEC__ONLY-blue?style=flat-square" alt="SPEC_ONLY">
<img src="https://img.shields.io/badge/IN__PROGRESS-yellow?style=flat-square" alt="IN_PROGRESS">
<img src="https://img.shields.io/badge/IMPLEMENTED-brightgreen?style=flat-square" alt="IMPLEMENTED">
<img src="https://img.shields.io/badge/DRIFTED-orange?style=flat-square" alt="DRIFTED">
<img src="https://img.shields.io/badge/ABANDONED-red?style=flat-square" alt="ABANDONED">
</p>

| Status | Significado |
|--------|-------------|
| `NOT_STARTED` | Intenção existe mas nenhum código relacionado |
| `SPEC_ONLY` | Documentação completa, zero implementação |
| `IN_PROGRESS` | Implementação parcial, branch ativa, ou TODOs visíveis |
| `IMPLEMENTED` | Código completo, com testes |
| `DRIFTED` | Código existe mas divergiu da spec |
| `ABANDONED` | Branch obsoleta + implementação parcial |

### Phantom Features

<p>
<img src="https://img.shields.io/badge/alerta-phantom%20features-red?style=flat-square" alt="Phantom Alert">
</p>

Features marcadas IMPLEMENTED no código mas que ainda aparecem como Open em issue tracker - alta chance de trabalho duplicado iminente. Detectadas e sinalizadas em `.first-plan/09-features/INDEX.md`.

### Fontes consultadas

<p>
<img src="https://img.shields.io/badge/local-docs%2Fspecs-blue?style=flat-square" alt="Local docs">
<img src="https://img.shields.io/badge/MCP-jira--mm-orange?style=flat-square" alt="JIRA MCP">
<img src="https://img.shields.io/badge/MCP-github--work-black?style=flat-square" alt="GitHub MCP">
<img src="https://img.shields.io/badge/git-history-darkgreen?style=flat-square" alt="Git history">
</p>

- Documentação local (`docs/`, `specs/`, `requirements/`, `rfcs/`, README sections)
- JIRA (via MCP `jira-mm` se disponível)
- GitHub Issues e PRs (via MCP `github-work` se disponível)
- Git history (branches, commit messages)
- Comentários no código (`TODO: implement`, `PLANNED:`, `FIXME`)

---

## Living Layer

<p>
<img src="https://img.shields.io/badge/hook-PostToolUse-brightgreen?style=flat-square" alt="Hook">
<img src="https://img.shields.io/badge/regenera-no-red?style=flat-square" alt="No regen">
<img src="https://img.shields.io/badge/sinaliza-yes-blue?style=flat-square" alt="Signal only">
</p>

O hook `PostToolUse` monitora edits via Edit/Write/MultiEdit e marca seções afetadas do `.first-plan/` como stale automaticamente. **Não regenera** - apenas sinaliza. O usuário decide quando rodar `/first-plan:refresh`.

Mapeamento arquivo modificado -> seções afetadas:

| Arquivo modificado | Seções marcadas stale |
|--------------------|----------------------|
| Manifesto (go.mod, package.json) | `01-topology/stacks` |
| `cmd/`, entry points | `01-topology/architecture` |
| Handlers / routers | `01-topology/boundaries` |
| Dockerfile, CI configs | `01-topology/deployments` |
| Source code (>= 5 arquivos) | `01-topology/activity`, `02-conventions/*` |
| `pkg/`, `lib/`, `utils/` | `03-reuse/*` |
| Tests | `02-conventions/testing`, `05-risks/untested` |
| docs/, specs/ | `09-features/*` |

---

## Plan-First Workflow

<p>
<img src="https://img.shields.io/badge/protocol-plan--first-red?style=flat-square" alt="Plan-First">
<img src="https://img.shields.io/badge/gate-humano-yellow?style=flat-square" alt="Human gate">
<img src="https://img.shields.io/badge/fases-5-blue?style=flat-square" alt="5 phases">
</p>

Protocolo obrigatório com gate humano explícito:

```
Discovery -> Plan -> Approval -> Execution -> Report
```

### Fase 1 - Discovery

```bash
/first-plan:init
```

Resultado: `.first-plan/` populado. Subagents read-only executam discovery em contexto isolado, retornam findings estruturados que são escritos no projeto-alvo.

### Fase 2 - Plan

```bash
/first-plan:plan <descrição da feature>
```

Resultado: `.first-plan/07-state/plans/<slug>.md` com:

1. Verificação de duplicidade (consulta `09-features/`)
2. Mapeamento de reuse aplicável (`03-reuse/`)
3. Arquivos a criar/modificar com diff conceitual
4. Aderência aos padrões (`02-conventions/`)
5. Riscos e perguntas abertas
6. Critério de "feito" + out-of-scope explícito

**Pausa pedindo aprovação humana.**

### Fase 3 - Approval

Estado: `awaiting_approval` em `STATE.md`. Não executa nada. Usuário aprova com `/first-plan:execute` ou pede ajustes.

### Fase 4 - Execution

```bash
/first-plan:execute
```

Segue o plano à risca. Para se algo invalidar premissa - **não improvisa**. Atualiza STATE a cada passo.

### Fase 5 - Report

Gerado automaticamente em `.first-plan/07-state/reports/<slug>.md` com:

- O que foi feito
- O que foi reusado vs criado do zero (com justificativa)
- Desvios do plano (se houver)
- Riscos remanescentes
- Sugestões fora do escopo

---

## Philosophy

<p>
<img src="https://img.shields.io/badge/regras-7-red?style=flat-square" alt="7 rules">
<img src="https://img.shields.io/badge/inviolaveis-yes-darkred?style=flat-square" alt="Inviolable">
</p>

### 7 Regras invioláveis

1. **Reuse first** - Antes de criar, verificar `.first-plan/03-reuse/INDEX.md`. Criação do zero exige justificativa explícita
2. **A verdade do projeto está no projeto** - Sem importar best practices externas. Se o projeto faz feio mas consistente, seguir o feio
3. **Sem dependências novas** - Usar apenas o que já existe em manifestos. Adicionar lib exige aprovação separada
4. **Consistência > elegância** - Refatoração não está no escopo a menos que solicitada. Sugestões vão para "fora do escopo" do report
5. **Criação do zero é exceção** - Permitida apenas quando não há precedente. Sempre justificar
6. **Acentuação completa em PT** - Comentários, docs, commits - tudo com acentuação correta
7. **Tipagem forte** - Respeitar nível de tipagem do projeto. Nada de `any`/`interface{}` se o projeto é estrito

### Confidence Scoring

<p>
<img src="https://img.shields.io/badge/threshold-0.7-yellow?style=flat-square" alt="Threshold 0.7">
<img src="https://img.shields.io/badge/range-0.0--1.0-blue?style=flat-square" alt="Range">
</p>

Cada finding tem `confidence: 0.0-1.0`. Threshold default: 0.7.

| Range | Significado |
|-------|-------------|
| `>= 0.9` | alta confiança, múltiplos sinais convergentes |
| `0.7-0.9` | boa confiança, sinal claro |
| `0.5-0.7` | confiança média, evidência circunstancial |
| `< 0.5` | baixa confiança, vira pergunta em `08-meta/questions.md` |

**O plugin não inventa - pergunta.** Quando confidence baixa, você é consultado via `/first-plan:ask`.

---

## System Requirements

<p>
<img src="https://img.shields.io/badge/Claude%20Code-required-orange?style=flat-square" alt="Claude Code">
<img src="https://img.shields.io/badge/Git-recommended-darkgreen?style=flat-square" alt="Git">
<img src="https://img.shields.io/badge/bash-required-black?style=flat-square" alt="bash">
<img src="https://img.shields.io/badge/MCPs-optional-lightgrey?style=flat-square" alt="MCPs">
</p>

- **Claude Code**: versão recente com plugin support
- **Git**: para Git Intelligence (opcional - se ausente, seções correlatas ficam vazias com nota)
- **bash**: hooks usam bash (Linux/macOS/WSL2)
- **MCPs opcionais** (melhoram cobertura):
  - `jira-mm` - reconciliation contra issues do JIRA
  - `github-work` - reconciliation contra issues/PRs do GitHub

Stack-agnostic - **não requer** runtimes específicos das stacks analisadas. O plugin lê código mas não executa.

---

## Configuration

O plugin não requer configuração inicial - convenções são descobertas no `init`. Customizações opcionais:

### Threshold de confidence

Edite o frontmatter de `.first-plan/08-meta/confidence.md`:

```yaml
---
threshold: 0.7    # ajustar para 0.6 (mais permissivo) ou 0.8 (mais rigoroso)
---
```

### Excluir paths do hook

Para evitar que mudanças em paths específicos disparem invalidação, edite `hooks/invalidate-cache.sh` ou adicione regra no `.gitignore` do projeto.

### Cache TTL

Por padrão git intelligence cacheada por 24h em `08-meta/cache.json`. Para forçar atualização:

```bash
/first-plan:refresh --all
```

---

## Development

### Estrutura do plugin

```
first-plan/
├── .claude-plugin/plugin.json       manifesto
├── commands/                        14 slash commands
├── skills/                          15 skills (1 protocol + 1 lens-engine + 8 lenses + 5 advanced)
├── agents/                          3 subagents (discovery, reconciliation, pattern)
├── hooks/                           hooks.json + invalidate-cache.sh
├── templates/                       41 templates copiados pro .first-plan/ no init
└── README.md
```

### Adicionar nova stack lens

1. Criar `skills/lens-<stack>/SKILL.md` seguindo contrato em `skills/lens-engine/SKILL.md`
2. Adicionar entrada na tabela de detecção em `skills/lens-engine/SKILL.md`
3. Sem outras mudanças necessárias

### Testar localmente

```bash
/plugin marketplace add /caminho/local/first-plan
/plugin install first-plan@first-plan
cd /algum/projeto
/first-plan:init
```

Para iterar, edite os arquivos do plugin e rode `/plugin reload` (ou reinicie Claude Code).

---

## Troubleshooting

### `.first-plan/` não foi criado após `/first-plan:init`

- Verifique se o diretório atual é um projeto válido (tem manifesto)
- Verifique se o plugin está instalado: `/plugin list`
- Veja log do subagent na saída do init para erros de discovery

### Hook não invalida seções após edits

- Verifique permissões: `chmod +x hooks/invalidate-cache.sh`
- Verifique log: `tail -f ~/.first-plan-hook.log`
- Verifique que `.first-plan/` existe no diretório do projeto

### Discovery muito lento em monorepo grande

- Discovery aplica amostragem automática para projetos > 1000 arquivos
- Use `/first-plan:refresh <seção>` para refreshar apenas uma seção
- Aumente `time_budget_minutes` no `init` se necessário

### Falsos positivos em DRIFTED

- Edite `.first-plan/09-features/<slug>.md` manualmente para corrigir status
- O plugin respeita edições manuais até o próximo refresh dessa feature

### Confidence média baixa

- Indica projeto com padrões inconsistentes ou em transição
- Veja `08-meta/questions.md` - responda perguntas e refresh
- Considere documentar convenções em CLAUDE.md para próximos discoveries

---

## Contributing

Contribuições são bem-vindas. Áreas de impacto:

- **Novas stack lenses** - Elixir, OCaml, Haskell, Scala, Clojure, Zig
- **Melhorias nos subagents** - especialmente reconciliation-auditor
- **Otimizações de performance** em monorepos
- **Integração com mais MCPs** (Linear, Asana, Notion)
- **v2 features** - co-change graph, migration tracker, decision archeology

Workflow:

1. Fork o repositório
2. Crie branch de feature
3. Implemente seguindo as próprias convenções do plugin (veja `skills/protocol/SKILL.md`)
4. Atualize documentação
5. Submeta Pull Request

---

## Roadmap

<p>
<img src="https://img.shields.io/badge/v0.4.1-current-brightgreen?style=flat-square" alt="v0.4.1 current">
<img src="https://img.shields.io/badge/v0.5.0-next-blue?style=flat-square" alt="v0.5.0 next">
<img src="https://img.shields.io/badge/v1.0-vision-lightgrey?style=flat-square" alt="v1.0 vision">
</p>

### Shipped

#### v0.1.0 - Initial release

- Discovery Layer completa (00-09)
- Spec-Code Reconciliation com phantom features detection
- Living Layer via hook PostToolUse
- 14 slash commands, 8 stack lenses, 3 read-only subagents

#### v0.2.0 - Cognitive Compiler Phase A+B

- **Provenance & Freshness Tracking** - schema com source/SHA/TTL/decay
- **Co-change Graph** - dependência de mudança via git history
- **Verification Loop** - lint/typecheck/tests post-execute
- **Rollback / Time Travel** - snapshots pre-execute

#### v0.3.0 - Native Rust Engine

- Binário `first-plan-engine` em Rust workspace
- Subcommands `cochange` e `hash` (10-100x speedup)
- Cross-platform pre-built binaries (linux x86_64+arm64, windows)
- GitHub Actions CI/CD (lint, test, release)

#### v0.4.0 - Semantic Search via BM25

- Engine `index` + `search` subcommands
- Tokenização identifier-aware (snake_case + camelCase + UPPER_CASE)
- BM25 ranking sobre símbolos extraídos de Go/Rust/TS/Python/PHP
- Skill `semantic-reuse` com graceful fallback
- Latência <10ms, zero token Claude

#### v0.4.1 - ML Embeddings (current)

- Feature flag `--features=ml` opt-in
- `core::embeddings` com FastEmbedProvider (BGE-small, ONNX)
- Hybrid search combinando BM25 + cosine similarity
- CLI `--mode bm25|embed|hybrid` + `--alpha` tuning
- Auto-download de modelos em `~/.cache/first-plan/models/`

### Planejado

#### v0.5.0 - Tree-sitter AST + Bash + Wikilinks

- Substituir regex extraction por tree-sitter (parsing exato)
- Suporte a Bash, Ruby, Java, Kotlin, Swift, Elixir
- Engine subcommand `ast` (assinatura/scope/refs)
- ML build cross-platform (linux musl + macOS + windows)
- **Wikilinks `[[secao]]` Obsidian-compatible** entre arquivos do `.first-plan/`
  - Inspirado em OpenKB - transforma a camada em graph navegavel

#### v0.6.0 - LSP Integration + Watch mode

- Engine fala com gopls/pyright/typescript-language-server/rust-analyzer
- Tipos reais via `textDocument/references` (vs heurística)
- Substitui grep por symbol-level navigation
- **Engine subcommand `watch`** - refresh incremental contínuo
  - Inspirado em OpenKB - filesystem monitoring com auto-refresh debounced
  - Vai alem do hook PostToolUse atual (que so sinaliza)

#### v0.7.0 - Multi-Repo Awareness + Multi-format docs

- Skill `cross-repo-mapping`
- Detecção de calls cross-repo (OpenAPI, gRPC, schemas)
- Comando `/first-plan:blast-radius <símbolo>` cross-service
- Config `~/.first-plan/repos.yaml` registry de sister repos
- **Ingestao de documentos multi-formato** no reconciliation
  - Inspirado em OpenKB - markitdown integration para ler PDF/Word/PowerPoint specs
  - Critico em ambientes corp onde specs nao sao markdown

#### v0.8.0 - CI/CD State + Contradiction detection

- Lê `.github/workflows/`, `.gitlab-ci.yml` (sabe checks que rodam)
- Detecta flaky tests do histórico
- Tags/releases comparison: "merged but not shipped"
- **Deteccao de contradicoes cross-section**
  - Inspirado em OpenKB - flagging automatico de findings conflitantes
  - Exemplo: `02-conventions/naming.md` diz snake_case mas `06-rationale/dont.md` lista snake_case como anti-pattern
  - Novo comando `/first-plan:contradictions`

### Long-term Vision (v1.0)

Cognitive Infrastructure completa:

- **Bug Recurrence DB** - "este bug apareceu antes em #234, fixed em abc123"
- **Decision Archeology** - extrai why/because de commits/PRs/comments
- **Migration Tracker** - "47% migrado de logrus → slog"
- **Doc-Code Sync auditor**
- **Test-Code Drift detector**
- **Investigation Mode** - bug-hunt subagent
- **Onboarding Path Generator** (por papel)
- **Team Awareness** (Slack/Linear sync)
- **Schema-Aware Operations** (OpenAPI/GraphQL/Protobuf breaking change detection)
- **Multi-Tool AI Sync** (Cursor + Cody + Copilot consomem `.first-plan/`)

---

## License

<p>
<img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License MIT">
</p>

MIT License - veja [LICENSE](./LICENSE) para detalhes completos.

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
