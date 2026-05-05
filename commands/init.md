---
description: Compila contexto completo do projeto-alvo gerando .first-plan/. Roda Discovery (stacks, conventions, reuse, domain, risks), Spec-Code Reconciliation e Git Intelligence. Idempotente - pode rodar múltiplas vezes.
argument-hint: [--force]
allowed-tools: [Read, Glob, Grep, Bash, Write, Edit, Task]
---

# /first-plan:init

Compilação completa do contexto do projeto-alvo.

## Argumentos

`$ARGUMENTS` - opcionalmente `--force` para reconstruir do zero ignorando cache.

## Workflow

### Passo 1 - Pre-flight check

1. Verificar diretório atual é um projeto (tem manifesto, código, etc) - **não rodar em $HOME ou diretório vazio**
2. Verificar se ja existe `.first-plan/`:
   - Se existe E não tem `--force`: avisar usuário e oferecer `/first-plan:refresh` em vez
   - Se existe E tem `--force`: backup em `.first-plan/.backup-<timestamp>/` antes de prosseguir
3. Verificar git repo. Se sim, ativa Git Intelligence; se não, marcar seções correlatas como N/A.

### Passo 2 - Bootstrapping

Criar a estrutura `.first-plan/` copiando templates do plugin:

```bash
mkdir -p .first-plan/{00-mission,01-topology,02-conventions,03-reuse,04-domain,05-risks,06-rationale,07-state/{plans,reports,sessions},08-meta,09-features}
```

Copiar templates de `${CLAUDE_PLUGIN_ROOT}/templates/` para `.first-plan/`. Substituir placeholders básicos:
- `PLACEHOLDER_TIMESTAMP` -> ISO 8601 atual
- `PLACEHOLDER_ROOT` -> caminho absoluto do projeto
- `PLACEHOLDER_NAME` -> nome do projeto (de package.json/go.mod/etc, ou nome da pasta)
- `PLACEHOLDER_PHASE` -> `discovering`

Adicionar ao `.gitignore` do projeto se não estiver:
```
.first-plan/cache/
.first-plan/07-state/sessions/
.first-plan/.backup-*/
```

### Passo 3 - Discovery (subagent)

Spawnar `discovery-analyst` via Task tool:

```
Task(
  subagent_type="discovery-analyst",
  description="Phase 1 Discovery for first-plan",
  prompt="<contrato detalhado com project_root, lens_skills_available, time_budget_minutes=8>"
)
```

Receber findings estruturados.

### Passo 4 - Pattern Extraction (subagent)

Spawnar `pattern-archeologist` em paralelo (mesma mensagem, múltiplas Task tool calls):

```
Task(
  subagent_type="pattern-archeologist",
  description="Pattern extraction for conventions and rationale",
  prompt="<contrato com project_root, discovery_findings={preliminar do passo 3 ou null se rodando em paralelo}, categories=[naming, errors, testing, logging, di, security, do, dont, why]>"
)
```

Receber findings de patterns.

### Passo 5 - Reconciliation (subagent)

Spawnar `reconciliation-auditor`:

```
Task(
  subagent_type="reconciliation-auditor",
  description="Spec-Code Reconciliation matrix",
  prompt="<contrato com project_root, discovery_findings={passo 3}, mcp_available={lista}>"
)
```

Receber matrix de features × status × evidência.

### Passo 6 - Git Intelligence (inline, sem subagent)

Rodar comandos git read-only conforme skill `git-intelligence`:
- Activity heatmap (top arquivos/pastas)
- Hot zones / frozen zones
- Ownership por path
- In-flight (branches + PRs via MCPs ou gh CLI se disponível)

Coletar resultados.

### Passo 7 - Escrita dos arquivos

Pegar findings dos passos 3-6 e escrever em `.first-plan/`:

| Findings | Arquivo target |
|----------|----------------|
| stacks (discovery) | 01-topology/stacks.md |
| architecture (discovery) | 01-topology/architecture.md |
| boundaries (discovery) | 01-topology/boundaries.md |
| deployments (discovery) | 01-topology/deployments.md |
| activity (git) | 01-topology/activity.md |
| ownership (git) | 01-topology/ownership.md |
| naming (patterns) | 02-conventions/naming.md |
| errors (patterns) | 02-conventions/errors.md |
| testing (patterns) | 02-conventions/testing.md |
| logging (patterns) | 02-conventions/logging.md |
| di (patterns) | 02-conventions/di.md |
| security (patterns) | 02-conventions/security.md |
| reuse_index (discovery) | 03-reuse/INDEX.md + arquivos detalhe + search.json |
| domain (discovery) | 04-domain/{glossary,entities,flows}.md |
| risks (discovery) | 05-risks/{fragile,untested,magic,debt}.md |
| do (patterns) | 06-rationale/do.md |
| dont (patterns) | 06-rationale/dont.md |
| why (patterns) | 06-rationale/why.md |
| in_flight (git) | 07-state/in-flight.md |
| features (reconciliation) | 09-features/INDEX.md + 09-features/<slug>.md por feature |

### Passo 8 - Meta

Calcular e escrever:

`08-meta/coverage.md`:
- Por seção: estado (filled/empty), última atualização
- Pastas/arquivos não-mapeados

`08-meta/confidence.md`:
- Confidence por seção
- Confidence média geral

`08-meta/questions.md`:
- Perguntas abertas coletadas dos subagents

`08-meta/cache.json`:
- Hash de arquivos relevantes (manifestos + entry points + arquivos top contribuintes)
- timestamp do init

### Passo 9 - State

Atualizar `07-state/STATE.md`:
- `phase: discovered`
- `last_session_id: <gerado>`
- `last_updated: <agora>`

### Passo 10 - INDEX.md final

Atualizar `INDEX.md` da raiz `.first-plan/`:
- `overall_confidence` na frontmatter
- Resumo de cobertura
- Próxima ação sugerida (geralmente "responda perguntas em questions.md, depois /first-plan:plan <feature>")

### Passo 11 - Reportar

Mostrar ao usuário:
- Stacks detectadas
- Quantidade de items no Reuse Index
- Quantidade de features classificadas (e breakdown por status)
- Quantidade de perguntas abertas
- Confidence média
- Sugestão de próxima ação

Exemplo de saída:

```
.first-plan/ compilado com sucesso

Stacks detectadas: 3 (Go, TypeScript, Terraform)
Reuse Index: 47 items
Features classificadas: 23
  IMPLEMENTED: 12
  IN_PROGRESS: 3
  SPEC_ONLY: 5
  NOT_STARTED: 2
  DRIFTED: 1 (atenção!)
  ABANDONED: 0
Phantom features detectadas: 2 (alerta!)

Confidence média: 0.78
Perguntas abertas: 4

Próximas ações:
1. Revisar phantom features em .first-plan/09-features/INDEX.md (alerta!)
2. Responder perguntas em .first-plan/08-meta/questions.md
3. Quando pronto, /first-plan:plan <feature>
```

## Tratamento de erros

- Subagent timeout: retornar findings parciais com nota
- Falha em discovery: oferecer rerun de seção específica
- Conflito com `.first-plan/` existente sem `--force`: orientar usuário

## Após sucesso

- O hook `PostToolUse` está ativo - edits posteriores marcam seções como stale automaticamente
- Use `/first-plan:status` para ver estado atual
- Use `/first-plan:refresh` quando código mudar
