---
generated_at: PLACEHOLDER_TIMESTAMP
project_root: PLACEHOLDER_ROOT
project_name: PLACEHOLDER_NAME
first_plan_version: 0.1.0
overall_confidence: 0.0
---

# `.first-plan/` - Camada compilada de contexto

Esta pasta é a representação intermediária (IR) do projeto, otimizada para IA. **Nao é documentação humana** - é fonte de verdade para Claude Code seguir padrões existentes do projeto sem reinventar nem reimplementar.

## Entry points

Comece pela seção que corresponde a sua intenção:

| Intenção | Arquivos a ler |
|----------|----------------|
| Entender propósito do projeto | `00-mission/` |
| Entender topologia / arquitetura / stacks | `01-topology/` |
| Seguir convenções de código | `02-conventions/` |
| Reusar componentes/utils existentes | `03-reuse/INDEX.md` |
| Entender domínio | `04-domain/` |
| Identificar áreas frágeis ou debt | `05-risks/` |
| Entender o porquê de decisões arquiteturais | `06-rationale/` |
| Saber o estado atual do trabalho IA | `07-state/STATE.md` |
| Saber o que tem certeza vs duvida | `08-meta/confidence.md` + `08-meta/questions.md` |
| Saber o que ja foi implementado | `09-features/INDEX.md` |

## Regras invioláveis (extraído de `02-conventions/` e `06-rationale/`)

PLACEHOLDER_INVIOLAVEIS

## Próxima ação sugerida

PLACEHOLDER_NEXT_ACTION

## Cobertura

PLACEHOLDER_COVERAGE_SUMMARY

## Confiança média

PLACEHOLDER_CONFIDENCE_AVG

Veja `08-meta/coverage.md` para detalhes de cobertura por seção e `08-meta/questions.md` para perguntas abertas.

## Como atualizar

- Mudou código? Rode `/first-plan:refresh`. O hook `PostToolUse` ja marcou as seções afetadas como stale em `08-meta/coverage.md`.
- Quer recompilar do zero? Apague `.first-plan/cache.json` e rode `/first-plan:init` novamente.
