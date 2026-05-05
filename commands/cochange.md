---
description: Mostra arquivos que tendem a mudar junto com o path especificado, baseado em git history. Use antes de planejar mudanças para evitar PRs incompletos.
argument-hint: <path>
allowed-tools: [Read, Bash]
---

# /first-plan:cochange

Query do Co-change Graph.

## Argumentos

`$ARGUMENTS` - path de arquivo (ex: `internal/payment/charge.go`).

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/02-conventions/co-change.md` ou `.first-plan/08-meta/co-change.json` existe. Se não, recomendar `/first-plan:refresh 02-conventions`.

### Passo 2 - Lookup

Ler `08-meta/co-change.json`, filtrar pares onde `file_a == $ARGUMENTS` ou `file_b == $ARGUMENTS`.

Se cache stale (> 14 dias), rodar inline:

```bash
git log --since="180 days ago" --name-only --pretty=format:"COMMIT:%H" --no-merges | \
  awk -v target="$ARGUMENTS" '
    /^COMMIT:/ { commit=$0; next }
    /./ { files[NR]=$0; commit_files[commit] = commit_files[commit] " " $0 }
    END {
      for (c in commit_files) {
        if (commit_files[c] ~ target) {
          n = split(commit_files[c], arr, " ")
          for (i=1; i<=n; i++) if (arr[i] != target && arr[i] != "") count[arr[i]]++
          total++
        }
      }
      for (f in count) printf "%.2f %d %s\n", count[f]/total, count[f], f
    }
  ' | sort -rn | head -10
```

### Passo 3 - Apresentar

```
=== Co-change graph para internal/payment/charge.go ===

Total commits afetando este arquivo (180 dias): 18

Top co-changers:

| Arquivo | Ratio | Commits compartilhados | Strength |
|---------|-------|------------------------|----------|
| internal/payment/invoice.go | 0.94 | 17/18 | strong |
| internal/handler/payment.go | 0.83 | 15/18 | moderate |
| internal/payment/refund.go | 0.72 | 13/18 | moderate |
| internal/payment/test_helpers.go | 0.61 | 11/18 | weak |

Cluster detectado: payment-subsystem
  Outros membros: internal/payment/refund.go, internal/handler/payment.go

Implicação:
- Antes de mexer em charge.go, considerar tocar nos co-changers strong
- Se PR atual modifica apenas charge.go, alta chance de regressão em invoice.go
- Cluster payment-subsystem indica boundary de módulo - mudanças tipicamente afetam todos
```

### Passo 4 - Sem co-changers detectados

```
=== Co-change graph para internal/util/format.go ===

Total commits afetando este arquivo (180 dias): 3

Sem co-changers significativos (insufficient sample size: precisa >= 5 commits).

Possíveis razões:
- Arquivo novo (criado recentemente)
- Arquivo estável (raramente mudado)
- Frozen zone (sem manutenção ativa)

Considere:
- /first-plan:hot para verificar se está frozen
- /first-plan:owner internal/util/format.go para descobrir dono
```

### Passo 5 - Modes especiais

#### Cluster lookup
Se argumento é um cluster name (ex: `payment-subsystem`):

```
=== Cluster: payment-subsystem ===

Membros (4):
- internal/payment/charge.go
- internal/payment/invoice.go
- internal/payment/refund.go
- internal/handler/payment.go

Cohesion interna: 0.87 (alta)
Total commits no cluster: 23

Implicação: este é um módulo coeso real. Mudanças idealmente tocam vários arquivos juntos.
```

#### Multiple paths
Se múltiplos paths separados por espaço:

```bash
/first-plan:cochange internal/payment/charge.go internal/payment/invoice.go
```

Mostra co-changers da intersecção (arquivos que mudam quando AMBOS mudam).

## Cuidados

- Co-change != dependência lógica. Pode ser apenas correlação histórica.
- Em projetos novos (< 50 commits no path), confiabilidade é baixa.
- Lockfiles e generated files são filtrados automaticamente (ver skill `co-change-analysis`).
- Refresh recomendado a cada 14 dias ou após eventos grandes (refator, migration).
