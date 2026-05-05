---
description: Mostra a cadeia completa de proveniência de um finding - source, commit SHA, age, confidence atual com decay, lifecycle status. Útil para auditar de onde veio uma afirmação na camada compilada.
argument-hint: <finding-id>
allowed-tools: [Read, Glob, Grep, Bash]
---

# /first-plan:provenance

Auditoria de proveniência de findings.

## Argumentos

`$ARGUMENTS` - finding ID (ex: `F-go-errors-001`) ou padrão de busca (ex: `F-go-*`).

## Workflow

### Passo 1 - Buscar finding

Grep por `finding_id: <ID>` em todos arquivos `.first-plan/`:

```bash
grep -rln "finding_id: $ARGUMENTS" .first-plan/
```

Se não encontrar match exato, tentar como prefix.

### Passo 2 - Extrair frontmatter completo

Ler arquivo e parsear o block YAML do finding.

### Passo 3 - Calcular confidence current

Aplicar decay rule da skill `provenance-tracker`:

```
age_days = (now - extracted_at).days
multiplier = decay_curve_lookup(age_days)
current = initial * multiplier
```

### Passo 4 - Apresentar

```
=== Provenance Report ===

Finding: F-go-errors-001
Type: pattern
Section: 02-conventions/errors
Status: active

Source:
  Type: code
  Location: internal/service/user.go:47
  Commit SHA: abc1234 (HEAD: def5678 - 12 commits ahead)
  Extracted from:
    - internal/service/user.go
    - internal/service/order.go
    - internal/service/payment.go (3 files total)

Timeline:
  Extracted at: 2026-04-15T10:00:00Z (20 days ago)
  Last revalidated: never
  TTL expires at: 2026-05-15T10:00:00Z (10 days remaining)

Confidence:
  Initial: 0.95
  Current: 0.85 (decayed - 30-90 days range)
  Signals used:
    - "12 occurrences"
    - "consistent across all services"

Lifecycle:
  Status: active
  Superseded by: none
  Supersedes: none
  Invalidated: no

Health: HEALTHY
  - Above threshold (0.85 > 0.7)
  - Within TTL (10 days left)
  - Source still exists in codebase

Recommendations:
  - Acceptable to use as basis for new code
  - Consider /first-plan:refresh 02-conventions/errors before deadline
```

### Passo 5 - Casos especiais

#### Finding superseded

```
Status: SUPERSEDED
Superseded by: F-go-errors-007 at 2026-05-01
Reason: New pattern emerged using errors.Join (Go 1.20+)

DO NOT USE THIS FINDING. Reference F-go-errors-007 instead.
View it: /first-plan:provenance F-go-errors-007
```

#### Finding invalidated

```
Status: INVALIDATED
Invalidated at: 2026-04-20
Reason: Manual mark - team decided this pattern was incorrect

DO NOT USE. See questions.md Q4 for resolution.
```

#### Finding expired

```
Status: EXPIRED
Expired at: 2026-05-15
Current confidence: 0.45 (below 0.5 threshold)

This finding was archived to .first-plan/08-meta/expired.md.
Run /first-plan:refresh to re-extract this section if still relevant.
```

### Passo 6 - Mode batch

Se argumento é wildcard (ex: `F-go-*`), listar todos matches em formato tabular curto:

```
=== Findings matching F-go-* ===

ID                       Type      Status        Confidence    Section
F-go-errors-001          pattern   active        0.85          02-conventions/errors
F-go-errors-007          pattern   active        0.95          02-conventions/errors
F-go-errors-001          pattern   superseded    -             02-conventions/errors (archived)
F-go-context-001         pattern   active        0.92          02-conventions/errors

Use /first-plan:provenance <id> para detalhes individuais.
```

## Cuidados

- Não modificar findings via este command (é read-only). Para alterar, editar arquivos diretamente ou rodar `/first-plan:refresh`.
- Se finding usa MCP source (jira/github), confidence pode estar baixa porque MCP esteve offline - mencionar isso.
- Sempre mostrar `commit_sha` extraído vs HEAD - se mudou muito (>20 commits), recomendar refresh.
