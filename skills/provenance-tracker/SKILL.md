---
name: first-plan-provenance-tracker
description: Skill que define o schema de proveniência e o sistema de freshness tracking para findings em .first-plan/. Use durante Discovery e refresh para registrar source (file:line@SHA), TTL, confidence inicial e cadeia de supersedeção. Aplica decay de confidence baseado em idade.
version: 0.2.0
---

# Provenance & Freshness Tracker

Cada finding em `.first-plan/` precisa de **rastreabilidade completa** - de onde veio, quando foi extraído, quão fresca é a info, quando expira, e o que substitui.

## Por que existe

Sem provenance, fatos viram lendas. Esta skill garante que toda afirmação em `.first-plan/` tem cadeia auditável até o código fonte. Isso elimina:

- **Context poisoning** - fatos stale interpretados como atuais
- **Hallucinated facts** - findings sem fonte verificável
- **Untrackable drift** - quando um finding ficou errado e por quê
- **No-op refreshes** - regerar tudo quando só uma coisa mudou

## Schema de proveniência

Cada item em `.first-plan/` (em qualquer arquivo) ganha campos no frontmatter ou inline (YAML/Markdown):

```yaml
---
finding_id: F-<unique>            # identificador estavel (ex: F-go-errors-001)
type: pattern | fact | rule | risk | feature | reuse_item
section: <section path>           # ex: 02-conventions/errors

source:                           # de onde veio
  type: code | doc | git | mcp | inferred
  location: path:line             # arquivo:linha
  commit_sha: <SHA>               # commit em que foi observado
  extracted_from:                 # contexto adicional
    - <file_path>
    - <file_path>

extracted_at: 2026-05-04T22:00:00Z
extracted_by: discovery-analyst | pattern-archeologist | reconciliation-auditor | manual

confidence:
  initial: 0.95                   # confiança no momento da extração
  current: 0.92                   # após decay (calculado)
  signals_used:                   # o que sustenta o score
    - "10+ occurrences in current code"
    - "documented in CLAUDE.md"
    - "consistent across modules"

ttl:
  days: 30                        # validade default
  expires_at: 2026-06-03T22:00:00Z
  decay_curve: linear | step | none

lifecycle:
  status: active | superseded | invalidated | expired
  superseded_by: F-<id>           # se substituido por finding mais novo
  superseded_at: <timestamp>
  invalidated_reason: <text>      # se marcado invalido manualmente
---
```

## Tipos de finding

| Type | Descrição | Onde aparece |
|------|-----------|--------------|
| `pattern` | Convenção/padrão extraído | `02-conventions/*` |
| `fact` | Afirmação concreta sobre o projeto | qualquer seção |
| `rule` | Regra inferida (do/dont) | `06-rationale/*` |
| `risk` | Risco identificado | `05-risks/*` |
| `feature` | Feature do projeto (Spec-Code) | `09-features/*` |
| `reuse_item` | Item reusável catalogado | `03-reuse/*` |

## Confidence decay

Findings perdem confidence com o tempo se não revalidados:

| Idade | Confidence multiplier |
|-------|----------------------|
| < 7 dias | 1.00 (sem decay) |
| 7-30 dias | 0.95 |
| 30-90 dias | 0.85 |
| 90-180 dias | 0.70 |
| > 180 dias | 0.50 |

Fórmula:
```
current_confidence = initial_confidence * age_multiplier
```

Confidence `current` é recalculada a cada `/first-plan:status` ou `/first-plan:refresh`.

Se `current < 0.5`, o finding é marcado como `expired` e movido para `08-meta/expired.md` (não deletado, preservado para auditoria).

## TTL strategy por tipo

| Type | TTL default | Razão |
|------|-------------|-------|
| `pattern` | 30 dias | Convenções podem mudar com refator |
| `fact` | 14 dias | Fatos sobre código mudam rápido |
| `rule` | 60 dias | Regras são mais estáveis |
| `risk` | 30 dias | Risco pode ser resolvido |
| `feature` | 7 dias | Status de implementação muda muito |
| `reuse_item` | 30 dias | Refators podem mudar API |

Usuário pode override via campo `ttl.days` no finding.

## Supersedeção

Quando um finding novo torna outro obsoleto:

```yaml
# Finding antigo
finding_id: F-go-errors-001
lifecycle:
  status: superseded
  superseded_by: F-go-errors-007
  superseded_at: 2026-05-15T10:30:00Z
```

```yaml
# Finding novo
finding_id: F-go-errors-007
supersedes:
  - F-go-errors-001
  - F-go-errors-003
```

Findings superseded permanecem no arquivo (apenas marcados) por 30 dias, depois movidos para `08-meta/superseded-archive.md`.

## Source types em detalhe

### `code`
Extraído por leitura direta do código.

```yaml
source:
  type: code
  location: internal/auth/jwt.go:47
  commit_sha: abc1234
  extracted_from:
    - internal/auth/jwt.go
    - internal/middleware/auth.go
```

### `doc`
Extraído de documentação local.

```yaml
source:
  type: doc
  location: docs/auth.md#jwt-flow
  commit_sha: abc1234
```

### `git`
Inferido do git history.

```yaml
source:
  type: git
  location: git log --since="30 days"
  commit_sha: HEAD
  extracted_from:
    - "47 commits affecting internal/payment/"
```

### `mcp`
Obtido via MCP externo.

```yaml
source:
  type: mcp
  location: jira-mm:PROJ-456
  extracted_at: 2026-05-04T22:00:00Z
```

### `inferred`
Conclusão derivada (não diretamente observada).

```yaml
source:
  type: inferred
  location: cross-cutting analysis
  extracted_from:
    - 02-conventions/errors.md (uses errors.Wrap)
    - 02-conventions/logging.md (uses slog)
  inference: "logging+errors pattern indicates structured observability stack"
```

## Operações

### Criar novo finding

Sempre incluir todos os campos obrigatórios do schema. Se não souber `commit_sha`, usar `git rev-parse HEAD` no momento da extração.

### Renovar (revalidar)

Quando `/first-plan:refresh` reanalizá área:

```yaml
extracted_at: 2026-05-04T22:00:00Z   # mantém original
revalidated_at: 2026-05-15T10:00:00Z  # novo campo
ttl:
  expires_at: 2026-06-14T10:00:00Z   # estendido
```

### Invalidar manualmente

Usuário pode marcar finding errado:

```yaml
lifecycle:
  status: invalidated
  invalidated_at: 2026-05-15T10:00:00Z
  invalidated_reason: "Padrão anterior estava errado - verificar com PR #42"
```

### Marcar superseded (durante refresh)

Quando finding novo emerge que substitui antigo, marcar ambos com cross-reference.

## Implicações para outras skills

- **discovery-analyst**: deve emitir todos findings com schema completo
- **pattern-archeologist**: idem
- **reconciliation-auditor**: idem para features
- **plan-emission**: ao consultar findings, verificar `lifecycle.status` e `confidence.current` (não usar superseded/expired/invalidated)
- **/first-plan:refresh**: recalcula `confidence.current` e marca expired/superseded conforme regras
- **/first-plan:status**: mostra count de findings active/superseded/expired

## Output

Schema aplicado **em todos os arquivos do `.first-plan/`** que contêm findings. Para arquivos como `02-conventions/errors.md` que tinham padrões ad-hoc, agora cada padrão é uma entrada YAML com schema completo.

Exemplo `02-conventions/errors.md` v2.0:

```markdown
---
section: conventions/errors
generated_at: 2026-05-04T22:00:00Z
findings_count: 3
active: 3
superseded: 0
expired: 0
average_confidence_initial: 0.95
average_confidence_current: 0.92
---

# Convenções de errors

## F-errors-001 - Wrapping com pkg/errors

```yaml
finding_id: F-errors-001
type: pattern
source:
  type: code
  location: internal/service/user.go:47
  commit_sha: abc1234
extracted_at: 2026-05-04T22:00:00Z
extracted_by: pattern-archeologist
confidence:
  initial: 0.95
  current: 0.92
  signals_used:
    - "12 occurrences"
    - "consistent across all services"
ttl:
  days: 30
  expires_at: 2026-06-03T22:00:00Z
lifecycle:
  status: active
```

**Padrão:**
\```go
return errors.Wrap(err, "failed to save user")
\```

**Visto em:** internal/service/user.go:47, internal/service/order.go:89, ...
```
