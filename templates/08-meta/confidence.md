---
section: meta/confidence
generated_at: PLACEHOLDER_TIMESTAMP
overall: 0.0
threshold: 0.7
---

# Confidence Map

Confidence por seção. Findings com confidence < threshold (0.7 default) viram entradas em `questions.md`.

## Por seção

| Seção | Confidence | Notas |
|-------|------------|-------|
| 00-mission/purpose | 0.0 | PLACEHOLDER |
| 00-mission/stakeholders | 0.0 | PLACEHOLDER |
| 01-topology/stacks | 0.0 | PLACEHOLDER |
| 01-topology/architecture | 0.0 | PLACEHOLDER |
| 01-topology/boundaries | 0.0 | PLACEHOLDER |
| 01-topology/deployments | 0.0 | PLACEHOLDER |
| 01-topology/activity | 0.0 | PLACEHOLDER |
| 01-topology/ownership | 0.0 | PLACEHOLDER |
| 02-conventions/naming | 0.0 | PLACEHOLDER |
| 02-conventions/errors | 0.0 | PLACEHOLDER |
| 02-conventions/testing | 0.0 | PLACEHOLDER |
| 02-conventions/logging | 0.0 | PLACEHOLDER |
| 02-conventions/di | 0.0 | PLACEHOLDER |
| 02-conventions/security | 0.0 | PLACEHOLDER |
| 03-reuse | 0.0 | PLACEHOLDER |
| 04-domain/glossary | 0.0 | PLACEHOLDER |
| 04-domain/entities | 0.0 | PLACEHOLDER |
| 04-domain/flows | 0.0 | PLACEHOLDER |
| 05-risks | 0.0 | PLACEHOLDER |
| 06-rationale | 0.0 | PLACEHOLDER |
| 09-features | 0.0 | PLACEHOLDER |

## Como interpretar

- `>= 0.9` - alta confiança, multiple sinais convergentes
- `0.7-0.9` - boa confiança, sinal claro mas com possíveis edge cases
- `0.5-0.7` - confiança média, evidência circunstancial
- `< 0.5` - baixa confiança, vira pergunta em `questions.md`
- `0.0` - não analisado ainda

## Fatores que aumentam confidence

- Múltiplos exemplos consistentes no codebase (>= 5 ocorrências)
- Documentação que confirma (CLAUDE.md, README, docstring)
- Consistência entre testes e código
- Padrão recente (últimos 3 meses de commits)

## Fatores que reduzem

- Apenas 1-2 exemplos
- Inconsistência entre arquivos (alguns fazem A, outros B)
- Ausência de testes para validar
- Padrão antigo, possivelmente em migração
