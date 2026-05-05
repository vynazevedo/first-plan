---
description: Mostra estado atual da camada .first-plan/ - fase, plano ativo, perguntas abertas, cobertura, confidence, e próxima ação sugerida.
argument-hint: [--verbose]
allowed-tools: [Read, Glob, Bash]
---

# /first-plan:status

Estado da camada `.first-plan/` no projeto-alvo.

## Argumentos

`$ARGUMENTS`:
- vazio: status resumido
- `--verbose`: status detalhado por seção

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/` existe. Se não:
```
.first-plan/ ainda não foi compilado neste projeto.

Para começar: /first-plan:init
```

### Passo 2 - Ler estado

Ler:
- `.first-plan/07-state/STATE.md` - phase, active_plan, last_session
- `.first-plan/08-meta/coverage.md` - cobertura por seção, stale list
- `.first-plan/08-meta/confidence.md` - confidence overall e por seção
- `.first-plan/08-meta/questions.md` - perguntas abertas
- `.first-plan/09-features/INDEX.md` - matriz de features (status breakdown)
- `.first-plan/INDEX.md` - frontmatter (overall_confidence, generated_at)

### Passo 3 - Renderizar status resumido (default)

```
first-plan status

Projeto: <project_name>
Compilado em: <generated_at> (<X dias atrás>)
Fase atual: <phase>

Cobertura: <X>% das seções preenchidas
Confidence média: <Y> (threshold: 0.7)
Perguntas abertas: <N>
Stale: <K> arquivo(s) (rodar /first-plan:refresh)

Features:
  IMPLEMENTED: <a>
  IN_PROGRESS: <b>  ← cuidado, evite duplicar
  SPEC_ONLY: <c>
  NOT_STARTED: <d>
  DRIFTED: <e>      ← <link>
  ABANDONED: <f>
  Phantom: <g>      ← <link>

Plano ativo: <slug ou "nenhum">

Próxima ação sugerida:
<determinada por fase + estado>
```

### Passo 4 - Mode --verbose

Adicionar:

```
=== Cobertura por seção ===
01-topology/stacks.md       confidence 0.91  ✓
01-topology/architecture.md confidence 0.85  ✓
01-topology/boundaries.md   confidence 0.72  ✓
01-topology/activity.md     confidence 1.00  ✓
01-topology/ownership.md    confidence 0.95  ✓
02-conventions/naming.md    confidence 0.88  ✓
02-conventions/errors.md    confidence 0.90  ✓
02-conventions/testing.md   confidence 0.65  ⚠ baixo
02-conventions/logging.md   confidence 0.50  ⚠ baixo - 2 perguntas abertas
... (todas as seções)

=== Stale (mudaram desde último refresh) ===
- src/payment/charge.go (modificado 2h atrás)
- src/auth/jwt.go (modificado ontem)

=== Perguntas abertas ===
Q1 (naming): camelCase vs snake_case em handlers
Q2 (logging): nível default em produção
Q3 (errors): wrapping mandatório ou opcional
Q4 (auth): JWT claim obrigatório vs opcional
```

### Passo 5 - Próxima ação

Determinada por fase + estado:

| Fase | Condição | Próxima ação sugerida |
|------|----------|------------------------|
| `discovered` | sem perguntas abertas | "Pronto para planejar feature: /first-plan:plan <feature>" |
| `discovered` | com perguntas abertas | "Responda perguntas: /first-plan:ask" |
| `discovered` | com stale | "Refresh recomendado: /first-plan:refresh" |
| `planning` | em andamento | "Plano em construção - aguarde" |
| `awaiting_approval` | - | "Plano <slug> aguarda aprovação. Veja .first-plan/07-state/plans/<slug>.md. Aprove com /first-plan:execute ou peça ajustes" |
| `executing` | - | "Execução em andamento - acompanhe progresso ou pause" |
| `paused` | - | "Execução pausada por <razão>. Veja STATE.md. Decisão: replanejar / abortar / continuar" |
| `done` | - | "Última feature concluída. Próxima: /first-plan:plan <feature>" |

### Passo 6 - Alertas

Se houver:
- Phantom features detectadas: alerta destacado
- DRIFTED features: alerta destacado
- Confidence média < 0.6: alerta de baixa confiança geral
- Perguntas abertas > 5: alerta de muitas ambiguidades

## Saída sempre concisa

Status default cabe em uma tela. --verbose pode ser longo, mas usuário escolheu.
