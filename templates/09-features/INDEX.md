---
section: features/index
generated_at: PLACEHOLDER_TIMESTAMP
total_features: 0
status_breakdown:
  not_started: 0
  spec_only: 0
  in_progress: 0
  implemented: 0
  drifted: 0
  abandoned: 0
---

# Spec-Code Reconciliation - Matriz de features

Para cada intenção declarada (em docs/, specs/, JIRA, GitHub issues, README), a matriz mostra **se ela já está implementada**, **parcial**, ou **não começou**. Antes de implementar feature nova, **consultar esta matriz primeiro**.

## Status possíveis

- `NOT_STARTED` - intenção existe mas nenhum código relacionado
- `SPEC_ONLY` - documentação completa, zero implementação
- `IN_PROGRESS` - implementação parcial, branch ativa, ou TODOs visíveis
- `IMPLEMENTED` - código completo, com testes
- `DRIFTED` - código existe mas divergiu da spec (spec desatualizada ou código tomou outra direção)
- `ABANDONED` - branch obsoleta + implementação parcial

## Matriz

| ID | Feature | Status | Confidence | Fonte da intenção | Evidência no código |
|----|---------|--------|------------|-------------------|---------------------|
| PLACEHOLDER_ID | PLACEHOLDER_FEATURE | PLACEHOLDER_STATUS | 0.0 | PLACEHOLDER_SOURCE | PLACEHOLDER_EVIDENCE |

## Detalhes por feature

Veja arquivos individuais em `09-features/<slug>.md` para evidência completa.

## Drift detectado (alerta!)

Features marcadas DRIFTED merecem atenção - código e spec divergiram:

PLACEHOLDER_DRIFT_LIST

## Phantom features (alerta!)

Features marcadas IMPLEMENTED que ainda aparecem em backlog/specs como pendentes:

PLACEHOLDER_PHANTOM_LIST
