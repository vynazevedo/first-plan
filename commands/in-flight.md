---
description: Mostra branches e PRs ativos no repositório. Use antes de planejar para evitar duplicação ou conflito com trabalho em andamento.
argument-hint: [--all|--mine]
allowed-tools: [Read, Bash]
---

# /first-plan:in-flight

Branches e PRs em movimento.

## Argumentos

`$ARGUMENTS`:
- vazio: mostra trabalho em flight (default)
- `--all`: inclui branches stale e PRs draft
- `--mine`: filtra para branches/PRs do usuário atual

## Workflow

### Passo 1 - Pre-flight

Verificar git repo. Verificar `.first-plan/07-state/in-flight.md` existe.

### Passo 2 - Render

Ler `07-state/in-flight.md` (snapshot mais recente). Se stale (> 24h), reroda inline a coleta.

### Passo 3 - Apresentar

```
Trabalho em flight (atualizado em <timestamp>)

Branches ativas (commit < 7 dias):
- feat/2fa            João Silva     2 dias    +127/-23 vs main   PR #42 aberto
- feat/audit-log      Maria Lima     1 dia     +89/-12 vs main    sem PR
- fix/payment-bug     Pedro Souza    3h        +5/-2 vs main      PR #45 aberto

Branches stale (7-30 dias):
- feat/csv-export     João Silva     12 dias   +45/-3 vs main     PR #38 (sem revisão há 8 dias)

Pull Requests abertos (<count>):
#42 [feat/2fa] Two-Factor Authentication                      João  ⏳ aguardando review
#45 [fix/payment] Fix retry on payment timeout                Pedro ✓ aprovado, aguarda merge
#38 [feat/csv-export] Add CSV export to /orders               João  ⚠ stale, sem update há 8 dias

WIP detectado:
- branch: wip/refactor-handlers (Maria, 5 dias)

Conflitos potenciais:
(se algum branch toca arquivos críticos do projeto, listar)
```

### Passo 4 - Modes

`--all`: inclui também branches abandoned (>30 dias), PRs draft.

`--mine`: filtra `git config user.email` e mostra apenas matches.

### Passo 5 - Sem MCP/gh

Se MCP github-work não disponível e gh CLI não instalado:

```
Trabalho em flight (limitado a dados locais - PRs externos não consultados)

Branches locais e remotas:
<lista via git for-each-ref>

Para PRs, instalar gh CLI ou habilitar MCP github-work.
```

## Atualização

Se `in-flight.md` está stale (> 24h):
- Avisar
- Oferecer rerun inline

```
07-state/in-flight.md está com 2 dias de idade.

Rodando refresh inline...
[output]

Para snapshot persistido: /first-plan:refresh 07-state
```
