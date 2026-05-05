---
section: state/in-flight
confidence: 0.0
generated_at: PLACEHOLDER_TIMESTAMP
---

# Trabalho em flight (branches + PRs)

Snapshot do que esta em movimento agora no projeto. **Verifique antes de planejar feature** para evitar duplicação.

## Branches locais

| Branch | Último commit | Diff vs main | Status inferido |
|--------|---------------|--------------|-----------------|
| PLACEHOLDER | PLACEHOLDER | PLACEHOLDER | PLACEHOLDER (active / stale / abandoned) |

## Branches remotas

PLACEHOLDER_REMOTE_BRANCHES

## Pull Requests abertos

(via `gh pr list` ou MCP github-work, se disponível)

| # | Título | Autor | Branch | Idade | Status |
|---|--------|-------|--------|-------|--------|
| PLACEHOLDER | PLACEHOLDER | PLACEHOLDER | PLACEHOLDER | PLACEHOLDER | PLACEHOLDER |

## WIP detectado

(branches/PRs com `wip`, `draft`, `[WIP]`, etc no nome ou título)

PLACEHOLDER_WIP

## Conflitos potenciais

PLACEHOLDER_POTENTIAL_CONFLICTS

## Implicações pra Claude

- Antes de criar plano de feature, **rodar `/first-plan:in-flight`** para evitar duplicar trabalho
- Se a feature pedida já tem PR aberto, recomendar continuar nele em vez de começar do zero
