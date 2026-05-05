---
description: Reverte as mudanças do último /first-plan:execute. Restaura snapshot do .first-plan/ e oferece git operations para reverter o working tree. Safety net para experimentação.
argument-hint: [--dry-run|--snapshot <id>]
allowed-tools: [Read, Glob, Bash]
---

# /first-plan:rollback

Reverte mudanças do último execute (ou snapshot específico).

## Argumentos

`$ARGUMENTS`:
- vazio: rollback do último execute
- `--dry-run`: mostra o que seria revertido sem aplicar
- `--snapshot <id>`: rollback para snapshot específico (ID do timestamp)

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/cache/snapshots/` existe e tem ao menos 1 snapshot.

### Passo 2 - Identificar snapshot alvo

```bash
ls -t .first-plan/cache/snapshots/ | head -5
```

Output exemplo:
```
2026-05-04T22-35-01-pre-execute-csv-export
2026-05-04T18-12-44-pre-execute-auth-jwt
2026-05-03T15-22-08-pre-execute-rate-limit
```

Default: snapshot mais recente. Se `--snapshot <id>`: usar esse.

### Passo 3 - Mostrar diff

```
=== Rollback preview ===

Snapshot alvo: 2026-05-04T22-35-01-pre-execute-csv-export
Idade: 23 minutos

Mudanças que serão revertidas em .first-plan/:
- 09-features/csv-export.md (criado pelo execute)
- 07-state/STATE.md (phase voltará para awaiting_approval)
- 07-state/reports/csv-export.md (será removido)
- 07-state/reports/csv-export-verification.md (será removido)

Mudanças no working tree (NÃO revertidas automaticamente):
- internal/handler/export.go (modified)
- internal/service/csv_writer.go (created)
- internal/service/csv_writer_test.go (created)

Para reverter working tree manualmente:
  git checkout HEAD -- internal/handler/export.go
  rm internal/service/csv_writer.go internal/service/csv_writer_test.go

OU se já commitou:
  git revert <commit-sha>

Continuar com rollback do .first-plan/? [s/N]
```

### Passo 4 - Aplicar (se confirmado)

```bash
# Backup do estado atual antes de rollback (snapshot do snapshot)
cp -r .first-plan .first-plan.before-rollback-$(date +%s)

# Restaurar snapshot
rm -rf .first-plan
cp -r .first-plan/cache/snapshots/<snapshot-id>/snapshot/.first-plan .first-plan
```

Atualizar STATE.md:
```yaml
phase: awaiting_approval  # ou phase pré-execute do snapshot
last_rollback_at: <ISO>
rolled_back_to: <snapshot-id>
```

### Passo 5 - Reportar

```
=== Rollback aplicado ===

.first-plan/ restaurado para: 2026-05-04T22-35-01-pre-execute-csv-export

Estado atual:
- phase: awaiting_approval
- active_plan: csv-export
- backup do estado anterior: .first-plan.before-rollback-1714942501/

Working tree:
- 3 arquivos do projeto AINDA estão modificados
- Reverter manualmente conforme instruções acima
- OU /first-plan:execute novamente para reaplicar plano

Próxima ação sugerida:
- Revisar plan em .first-plan/07-state/plans/csv-export.md
- Ajustar plano se necessário (descobriu algo novo?)
- Executar quando pronto
```

### Passo 6 - Mode --dry-run

Mostra preview (Passo 3) e termina sem aplicar nada.

### Modo --snapshot

```bash
/first-plan:rollback --snapshot 2026-05-03T15-22-08-pre-execute-rate-limit
```

Volta para snapshot específico (não o mais recente). Útil pra desfazer múltiplos executes consecutivos.

## Snapshots como cleanup

Snapshots antigos (> 30 dias) são automaticamente removidos no próximo init/refresh para economizar espaço.

Listar todos:
```bash
ls -la .first-plan/cache/snapshots/
```

Limpar manualmente:
```bash
find .first-plan/cache/snapshots/ -mindepth 1 -maxdepth 1 -mtime +30 -exec rm -rf {} \;
```

## Cuidados

- Rollback **não toca no working tree** automaticamente - é decisão sua
- Se já comitou as mudanças do execute, precisa `git revert` separadamente
- Snapshots ocupam espaço (~tamanho do `.first-plan/` cada). Limite recomendado: últimos 10
- Se rollback dá errado, há backup em `.first-plan.before-rollback-*` (cleanup manual)

## Quando NÃO usar rollback

- Se o execute foi bem-sucedido e você só quer ajustar - use `/first-plan:plan` para um novo
- Se quer voltar ao estado pre-init - use `/first-plan:init --force` (regenera tudo)
- Se snapshots estão corrompidos - delete `.first-plan/` e reinit
