---
name: first-plan-co-change-analysis
description: Skill que constrói o Co-change Graph - "quando arquivo X muda, qual outro arquivo geralmente muda junto?". Use durante Discovery e em /first-plan:cochange. Baseado em git history. Detecta arquivos co-dependentes que devem ser editados juntos para evitar PRs incompletos. v0.3.0+ usa engine nativo Rust quando disponível (10-100x mais rápido).
version: 0.3.0
---

# Co-change Graph

Mapeia dependências de mudança baseadas em git history. Resolve o problema "esqueci de atualizar o consumer" - causa #1 de PRs incompletos em projetos grandes.

## Engine acceleration (v0.3.0+)

Se o binário nativo `first-plan-engine` está disponível (ver skill `first-plan-engine-bootstrap`), invocar direto e parsear o JSON. **10-100x mais rápido** vs git log via shell + agregação em Claude.

```bash
# Detectar engine
ENGINE=""
for c in "${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine" "${HOME}/.local/bin/first-plan-engine" "$(command -v first-plan-engine 2>/dev/null)"; do
  [ -x "$c" ] && ENGINE="$c" && break
done

if [ -n "$ENGINE" ]; then
  # Engine path - rapido e exato
  "$ENGINE" cochange \
    --repo "$PWD" \
    --since 180 \
    --min-occurrences 5 \
    --min-ratio 0.5 \
    --output-json /tmp/cochange-${SESSION_ID}.json

  # Parsear /tmp/cochange-*.json (schema first-plan-cochange-v1)
  # Renderizar como markdown padrão em .first-plan/02-conventions/co-change.md
else
  # Fallback markdown (algoritmo abaixo)
fi
```

JSON output schema (`first-plan-cochange-v1`):
- `pairs[].file_a`, `file_b`, `co_change_ratio`, `shared_commits`, `total_a`, `total_b`, `strength`
- `clusters[].id`, `files`, `internal_cohesion`
- Meta: `engine_version`, `generated_at`, `repo_root`, `window_days`, `total_commits_analyzed`, `total_files_analyzed`, `elapsed_ms`

Se engine ausente, oferecer instalação via skill `first-plan-engine-bootstrap` antes de cair no fallback.

---

## Fallback markdown (sem engine)

## Conceito

Para cada par de arquivos (A, B), calcular:

```
co_change_ratio(A, B) = commits_modificando_ambos / commits_modificando_A
```

Se `co_change_ratio(A, B) >= 0.7`, B é um **co-changer forte** de A. Mexer em A sem tocar em B é red flag.

## Algoritmo

### Passo 1 - Coletar commits relevantes

```bash
git log --since="180 days ago" --name-only --pretty=format:"COMMIT:%H" --no-merges
```

Output: lista de commits e arquivos modificados em cada.

### Passo 2 - Construir matriz

Para cada commit, listar conjunto de arquivos `S_i = {f1, f2, ...}` modificados.

Para cada par `(A, B)` que aparece em algum `S_i`:
- `co_changes(A, B)` = quantos `S_i` contêm ambos
- `total_changes(A)` = quantos `S_i` contêm A
- `ratio(A, B) = co_changes(A, B) / total_changes(A)`

Filtrar:
- `total_changes(A) >= 5` (sinal estatístico mínimo)
- `co_change_ratio >= 0.5` (significativo)

### Passo 3 - Classificar força

| Ratio | Força | Significado |
|-------|-------|-------------|
| >= 0.9 | strong | Quase sempre mudam juntos |
| 0.7-0.9 | moderate | Frequentemente juntos |
| 0.5-0.7 | weak | Mudam juntos com frequência notável |
| < 0.5 | none | Não considerar co-changer |

### Passo 4 - Detectar clusters

Aplicar clustering simples (Union-Find) sobre pares com `ratio >= 0.7`:
- Arquivos no mesmo cluster são "módulo coeso de mudança"
- Útil para detectar boundaries reais (vs declarados)

## Output em `.first-plan/`

### `02-conventions/co-change.md`

```markdown
---
section: conventions/co-change
generated_at: <ISO>
window_days: 180
total_files_analyzed: 245
strong_pairs: 23
moderate_pairs: 67
clusters_detected: 8
---

# Co-change Graph

Pares de arquivos que tendem a mudar juntos, baseado em 180 dias de git history.

## Pares fortes (>= 0.9)

| Arquivo A | Arquivo B | Co-change ratio | Commits compartilhados |
|-----------|-----------|-----------------|------------------------|
| internal/payment/charge.go | internal/payment/invoice.go | 0.94 | 17/18 |
| internal/auth/jwt.go | internal/middleware/auth.go | 0.92 | 11/12 |
| ... |

## Pares moderados (0.7-0.9)

(tabela similar)

## Clusters detectados

### Cluster 1 - Payment subsystem
Arquivos:
- internal/payment/charge.go
- internal/payment/invoice.go
- internal/payment/refund.go
- internal/handler/payment.go

Indicação: módulo coeso real, mudanças tipicamente afetam todos.

### Cluster 2 - ...

## Implicações pra Claude

- Antes de mexer em arquivo X, verificar `/first-plan:cochange X`
- Se algum co-changer não estiver no plano, alertar antes de execute
- Clusters detectados podem indicar boundaries de módulo - útil pra refator
```

### `08-meta/co-change.json` (machine-readable)

```json
{
  "$schema": "first-plan-co-change-v1",
  "generated_at": "<ISO>",
  "window_days": 180,
  "pairs": [
    {
      "file_a": "internal/payment/charge.go",
      "file_b": "internal/payment/invoice.go",
      "co_change_ratio": 0.94,
      "shared_commits": 17,
      "total_a": 18,
      "total_b": 19,
      "strength": "strong"
    }
  ],
  "clusters": [
    {
      "id": "cluster-1",
      "name": "payment-subsystem",
      "files": [
        "internal/payment/charge.go",
        "internal/payment/invoice.go",
        "internal/payment/refund.go",
        "internal/handler/payment.go"
      ],
      "internal_cohesion": 0.87
    }
  ]
}
```

## Performance em projetos grandes

Em monorepos com 50k+ commits, o git log pode ser lento. Otimizações:

- Cap em `--since="180 days ago"` (já default)
- Limit `--max-count=5000` (commits mais recentes apenas)
- Cachear matriz em `08-meta/co-change.json` com TTL 7 dias
- Apenas recalcular se houve >100 commits novos desde último cálculo

Em projetos < 1000 commits, full scan é trivialmente rápido (<10s).

## Falsos positivos comuns

- **Lockfiles**: `package-lock.json` muda toda vez que `package.json` muda - filtrar lockfiles
- **Generated files**: `*.gen.go`, `*.pb.go` - filtrar via gitignore patterns
- **Migration files**: novos arquivos a cada feature - excluir `migrations/` ou similar
- **Test files que sempre acompanham source**: legitimate co-change, manter

Filtros padrão excluem:
- `*.lock`, `*.lockb`, `package-lock.json`, `yarn.lock`, `Cargo.lock`, `go.sum`, `composer.lock`
- `*.min.js`, `*.bundle.js`
- `vendor/`, `node_modules/`, `target/`
- Arquivos com `.gen.`, `.pb.`, `_pb2.py`

## Integração com /first-plan:plan

Quando usuário roda `/first-plan:plan <feature>`:

1. Para cada arquivo no plano (a modificar ou criar), consultar co-change graph
2. Se algum co-changer com `ratio >= 0.7` não está no plano, listar como "potential missing co-change":

```markdown
## Riscos e ambiguidades

### Co-change alerts

- Vai modificar `internal/payment/charge.go` mas plano não menciona:
  - `internal/payment/invoice.go` (co-change 0.94 - strong)
  - `internal/handler/payment.go` (co-change 0.81 - moderate)

  Considerar adicionar ao plano. Se intencional não tocar, justificar.
```

## Confidence

Co-change findings sempre têm:
- `confidence.initial`: baseado em `total_changes(A)` (mais commits = mais confiável)
- `signals_used`: ratio + sample size + window
- `ttl.days`: 14 (refresh frequente porque git history muda)

```yaml
finding_id: F-cochange-payment-001
type: pattern
source:
  type: git
  location: git log --since="180 days ago"
  commit_sha: <HEAD>
  extracted_from:
    - "17 commits affecting both files"
extracted_at: <ISO>
extracted_by: discovery-analyst
confidence:
  initial: 0.88
  signals_used:
    - "ratio: 0.94"
    - "shared_commits: 17"
    - "total_a: 18 (sufficient sample)"
ttl:
  days: 14
lifecycle:
  status: active
data:
  file_a: internal/payment/charge.go
  file_b: internal/payment/invoice.go
  co_change_ratio: 0.94
  strength: strong
```
