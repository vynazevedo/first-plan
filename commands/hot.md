---
description: Mostra hot zones do projeto - áreas com mais commits recentes. Útil para entender onde o time está investindo agora.
argument-hint: [--days N]
allowed-tools: [Read, Bash]
---

# /first-plan:hot

Heatmap de atividade.

## Argumentos

`$ARGUMENTS`:
- vazio: window de 30 dias (default)
- `--days N`: window customizado (e.g., `--days 7`, `--days 90`)

## Workflow

### Passo 1 - Pre-flight

Verificar git repo. Verificar `.first-plan/01-topology/activity.md` existe.

### Passo 2 - Render

Para window default (30 dias): ler `activity.md`.

Para window customizado: rerun inline:
```bash
git log --since="N days ago" --name-only --pretty=format: | grep -v '^$' | sort | uniq -c | sort -rn | head -20
```

### Passo 3 - Apresentar

```
Hot zones - últimos <N> dias

Top 20 arquivos:
  47 internal/payment/charge.go         (refator em curso?)
  31 internal/handler/order.go          
  28 src/components/Cart.tsx            
  ...

Top 20 pastas:
  127 internal/payment/                 hot zone ⚠
   89 src/components/                   hot zone ⚠
   67 internal/handler/                 
  ...

Hot zones (>= 10 commits):
- internal/payment/    - 127 commits (intensa atividade)
- src/components/      - 89 commits (provavelmente UI sendo polida)

Frozen zones (sem commits > 180 dias):
- internal/legacy/     - silencioso ha 8 meses
- pkg/oldauth/         - silencioso ha 14 meses (candidato dead code?)

Implicações:
- Padrões em hot zones podem estar em transição - olhar últimos commits, não histórico antigo
- Frozen zones têm padrões antigos e estáveis
- Mexer em hot zones = alta chance de conflito com PRs em flight (ver /first-plan:in-flight)
```

### Passo 4 - Conexão com outras camadas

Sugestões automáticas:
- Se algum hot zone tem feature DRIFTED associada: alertar
- Se algum hot zone tem ownership claro: mostrar dono
- Se algum hot zone tem PR em flight: linkar
