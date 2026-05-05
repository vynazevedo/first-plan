---
description: Atualiza incrementalmente .first-plan/ baseado nos arquivos marcados como stale pelo hook PostToolUse. Mais barato que /first-plan:init - só recompila o que mudou.
argument-hint: [section|--all]
allowed-tools: [Read, Glob, Grep, Bash, Write, Edit, Task]
---

# /first-plan:refresh

Atualização incremental da camada compilada.

## Argumentos

`$ARGUMENTS`:
- vazio: atualiza só seções marcadas como stale
- nome de seção (e.g., `01-topology`): força refresh dessa seção
- `--all`: força refresh completo (equivalente a `/first-plan:init --force` mas preserva backups)

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/` existe. Se não, orientar a rodar `/first-plan:init` primeiro.

### Passo 2 - Determinar escopo

1. Ler `.first-plan/08-meta/cache.json` para identificar arquivos com hash diferente (modificados desde último init/refresh)
2. Ler `.first-plan/08-meta/coverage.md` seção "Stale" para arquivos marcados pelo hook
3. Mapear arquivos modificados → seções afetadas:

| Arquivo modificado | Seções afetadas |
|--------------------|-----------------|
| Manifesto (go.mod/package.json/etc) | 01-topology/stacks |
| `cmd/`, entry points | 01-topology/architecture |
| Handlers / routers | 01-topology/boundaries |
| Dockerfile, .github/workflows | 01-topology/deployments |
| Qualquer código (>= 5 arquivos) | 01-topology/activity, 02-conventions/* |
| pkg/, lib/, utils/ | 03-reuse/* |
| Models, types | 04-domain/entities |
| Flows críticos (handlers de domínio) | 04-domain/flows |
| Tests | 02-conventions/testing, 05-risks/untested |
| Config files (lint, format) | 02-conventions/* |
| docs/, specs/ | 09-features/* |

Se `$ARGUMENTS` é nome de seção, force apenas essa.

### Passo 3 - Refresh por seção

Para cada seção a atualizar:

#### 01-topology/* (excluindo activity, ownership)
Spawnar `discovery-analyst` com escopo restrito (apenas reanalisar essa seção).

#### 01-topology/activity, 01-topology/ownership
Inline - rodar comandos git novamente.

#### 02-conventions/*, 06-rationale/*
Spawnar `pattern-archeologist` com `categories=[<seção específica>]`.

#### 03-reuse/*
Spawnar `discovery-analyst` com escopo `reuse-only`.

#### 04-domain/*
Spawnar `discovery-analyst` com escopo `domain-only`.

#### 05-risks/*
Spawnar `discovery-analyst` com escopo `risks-only`.

#### 07-state/in-flight
Inline - rodar gh CLI ou MCP github-work novamente.

#### 09-features/*
Spawnar `reconciliation-auditor` para reclassificar features afetadas.

### Passo 4 - Reescrever só os arquivos atualizados

Não tocar em arquivos cuja seção não foi refreshed.

### Passo 5 - Atualizar metadados

`08-meta/cache.json`:
- Atualizar hashes dos arquivos refreshed
- Limpar `stale_paths`
- Atualizar `last_refresh`

`08-meta/coverage.md`:
- Limpar lista "Stale"
- Atualizar timestamps das seções

`08-meta/confidence.md`:
- Recalcular confidence das seções afetadas

### Passo 6 - Reportar diff

Mostrar ao usuário:
- Quais seções foram refreshed
- Diff de cada (resumo curto: "stacks.md: 3 → 3 stacks (sem mudança)", "features/INDEX.md: 23 → 25 features (+2 IMPLEMENTED)", etc)
- Tempo gasto vs tempo de init completo

Exemplo:

```
Refresh completo em 47s (vs ~3min de init)

Seções atualizadas:
- 02-conventions/errors.md (padrão de wrapping mudou: agora usa errors.Join)
- 03-reuse/utils.md (+2 utils: ValidateCNPJ, FormatCurrencyBRL)
- 09-features/INDEX.md (1 feature mudou de IN_PROGRESS para IMPLEMENTED)

Sem mudanças relevantes em: 01-topology, 04-domain, 05-risks, 06-rationale, 07-state

Próxima ação sugerida: revisar diff em .first-plan/09-features/INDEX.md
```

## Quando usar refresh vs init

- **refresh:** mudanças incrementais, depois de implementar feature, depois de pull
- **init --force:** mudança estrutural grande, refator de framework, virada de stack

## Performance

- Cache hit (sem mudanças): < 2s
- Refresh leve (1-2 seções): 30-60s
- Refresh maior: 1-3min
- vs init completo: 2-8min
