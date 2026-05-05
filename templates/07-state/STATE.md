---
section: state/machine
phase: PLACEHOLDER_PHASE
last_updated: PLACEHOLDER_TIMESTAMP
last_session_id: PLACEHOLDER_SESSION_ID
active_plan: PLACEHOLDER_ACTIVE_PLAN_OR_NULL
---

# State Machine

Estado atual do trabalho IA neste projeto. Atravessa sessões.

## Fase atual

`PLACEHOLDER_PHASE`

Valores possíveis:
- `uninitialized` - .first-plan/ ainda nao existe (impossível ler este arquivo se for esse caso)
- `discovered` - Discovery completa, pronto para planejar
- `planning` - plano de feature em construção
- `awaiting_approval` - plano aguardando aprovação humana
- `executing` - execução em curso
- `paused` - execução pausada por invalidação ou bloqueio
- `done` - feature mais recente concluída, contexto pronto para próxima

## Plano ativo

PLACEHOLDER_ACTIVE_PLAN_DETAILS

## Última sessão

- ID: PLACEHOLDER_SESSION_ID
- Início: PLACEHOLDER_START
- Fim: PLACEHOLDER_END
- Resumo: PLACEHOLDER_SUMMARY

## Sessões anteriores

Veja `sessions/` (efêmero, gitignored).

## Planos

Ativos: `plans/`
Concluídos: `reports/`

## Como retomar trabalho

Após reabrir o Claude Code, o entry point é este arquivo. Claude deve:
1. Ler `phase`
2. Se `awaiting_approval`: mostrar o plano em `plans/<active_plan>` e pedir decisão
3. Se `executing` ou `paused`: relatar último passo executado e proxima ação
4. Se `discovered`: pronto para `/first-plan:plan <feature>`
