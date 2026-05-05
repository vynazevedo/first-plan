---
description: Mostra a matriz de features × status × evidência consolidada em 09-features/INDEX.md. Filtragem por status disponível.
argument-hint: [status:IMPLEMENTED|IN_PROGRESS|SPEC_ONLY|NOT_STARTED|DRIFTED|ABANDONED]
allowed-tools: [Read, Glob, Grep]
---

# /first-plan:features

Matriz de Spec-Code Reconciliation.

## Argumentos

`$ARGUMENTS`:
- vazio: tabela completa
- `status:<X>`: filtrar por status específico
- `phantom`: apenas phantom features
- `drifted`: apenas drift detectado

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/09-features/INDEX.md` existe.

### Passo 2 - Carregar matriz

Ler INDEX.md + frontmatter + tabela.

### Passo 3 - Aplicar filtro

Se filtro aplicado, manter apenas linhas que casam.

### Passo 4 - Apresentar

Default:

```
Matriz de features (<total>)

IMPLEMENTED (<a>):
  F1  User Authentication via JWT       conf 0.91  evidência: internal/auth/jwt.go
  F3  Password Reset Flow               conf 0.85  evidência: internal/auth/reset.go
  ... 

IN_PROGRESS (<b>):
  F5  Two-Factor Authentication         conf 0.78  branch: feat/2fa, PR #42
  ...

SPEC_ONLY (<c>):
  F7  CSV Export                         conf 0.92  fonte: JIRA-456
  ...

NOT_STARTED (<d>):
  F9  Audit Log Replay                   conf 0.70  fonte: docs/audit.md
  ...

DRIFTED (<e>) ⚠
  F12 Email Validation                  conf 0.82  drift: regex permissiva vs spec estrita
  ...

ABANDONED (<f>):
  F15 Old Notification Center           conf 0.65  branch obsoleta há 4 meses
  ...

Phantom features ⚠ (<g>):
  F8  Pagination on /users               implementada mas JIRA-234 ainda Open

Detalhes individuais: .first-plan/09-features/<slug>.md
```

Filtrado:

```
Features com status=IMPLEMENTED (<count>)

F1  User Authentication via JWT          conf 0.91
    fonte: docs/auth.md
    evidência: internal/auth/jwt.go:1-150 + tests
    detalhes: .first-plan/09-features/user-authentication-jwt.md

F3  Password Reset Flow                  conf 0.85
    ...
```

### Passo 5 - Alertas

Sempre destacar:
- Phantom features (alerta forte)
- DRIFTED (alerta médio)
- IN_PROGRESS sem update há > 30 dias (sinal de abandonment iminente)

## Cuidados

- Se INDEX.md está stale, recomendar `/first-plan:refresh 09-features` antes de confiar plenamente.
