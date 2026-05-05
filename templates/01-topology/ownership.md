---
section: topology/ownership
confidence: 0.0
generated_at: PLACEHOLDER_TIMESTAMP
---

# Ownership inferido

Quem domina cada área do código (top contributor por path) baseado em `git log`/`git blame`. Útil para "pergunte ao @fulano" antes de mexer em algo crítico.

## Por pasta principal

| Pasta | Top contributor | % autoria | Co-autores notáveis |
|-------|-----------------|-----------|---------------------|
| PLACEHOLDER | PLACEHOLDER | PLACEHOLDER | PLACEHOLDER |

## Arquivos críticos sem ownership claro

(Arquivos modificados por > 10 pessoas sem contribuidor majoritário)

PLACEHOLDER_NO_CLEAR_OWNER

## Arquivos órfãos

(Última modificação > 1 ano e contribuidor original nao está mais ativo)

PLACEHOLDER_ORPHANS

## Implicações pra Claude

- Antes de fazer mudança grande em uma pasta, verificar quem domina (via `/first-plan:owner <path>`)
- Arquivos órfãos = alta chance de ter sido esquecido. Validar com humano se ainda é usado antes de assumir.
- Arquivos sem ownership claro = padrões podem ter divergido. Olhar os commits recentes mais que histórico antigo.
