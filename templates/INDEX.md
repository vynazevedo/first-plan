---
generated_at: PLACEHOLDER_TIMESTAMP
project_root: PLACEHOLDER_ROOT
project_name: PLACEHOLDER_NAME
first_plan_version: 0.5.0
overall_confidence: 0.0
---

# `.first-plan/` - Camada compilada de contexto

Esta pasta é a representação intermediária (IR) do projeto, otimizada para IA. **Nao é documentação humana** - é fonte de verdade para Claude Code seguir padrões existentes do projeto sem reinventar nem reimplementar.

## Navegação (wikilinks Obsidian-compatible)

A camada usa wikilinks `[[secao]]` estilo Obsidian. Abra esta pasta no Obsidian/Logseq pra navegar como graph. No GitHub web, os links são apenas texto - veja [Como navegar](#como-navegar) abaixo.

## Entry points

Comece pela seção que corresponde a sua intenção:

| Intenção | Arquivos a ler |
|----------|----------------|
| Entender propósito do projeto | [[00-mission/purpose]] |
| Entender stakeholders | [[00-mission/stakeholders]] |
| Entender topologia / arquitetura / stacks | [[01-topology/stacks]], [[01-topology/architecture]] |
| Entender boundaries (APIs, eventos, schemas) | [[01-topology/boundaries]] |
| Como o projeto deploya | [[01-topology/deployments]] |
| Hot zones (atividade recente) | [[01-topology/activity]] |
| Quem domina cada área | [[01-topology/ownership]] |
| Seguir convenções de naming | [[02-conventions/naming]] |
| Padrões de error handling | [[02-conventions/errors]] |
| Estilo de testes | [[02-conventions/testing]] |
| Padrões de logging | [[02-conventions/logging]] |
| Dependency injection | [[02-conventions/di]] |
| Padrões de segurança | [[02-conventions/security]] |
| Reusar componentes/utils existentes | [[03-reuse/INDEX]] |
| Componentes UI / módulos reusáveis | [[03-reuse/components]] |
| Utilities helper | [[03-reuse/utils]] |
| Tipos compartilhados | [[03-reuse/types]] |
| Hooks frontend | [[03-reuse/hooks]] |
| Glossário do domínio | [[04-domain/glossary]] |
| Entidades de domínio | [[04-domain/entities]] |
| Flows críticos | [[04-domain/flows]] |
| Áreas frágeis | [[05-risks/fragile]] |
| Áreas sem testes | [[05-risks/untested]] |
| Código clever / magic | [[05-risks/magic]] |
| Débito técnico | [[05-risks/debt]] |
| Padrões a seguir | [[06-rationale/do]] |
| Anti-padrões a evitar | [[06-rationale/dont]] |
| Por que decisões foram tomadas | [[06-rationale/why]] |
| Estado atual do trabalho IA | [[07-state/STATE]] |
| Branches/PRs em flight | [[07-state/in-flight]] |
| Cobertura por seção | [[08-meta/coverage]] |
| Confiança por finding | [[08-meta/confidence]] |
| Perguntas abertas pro humano | [[08-meta/questions]] |
| Matriz de features × status | [[09-features/INDEX]] |

## Regras invioláveis (extraído de [[02-conventions]] e [[06-rationale]])

PLACEHOLDER_INVIOLAVEIS

## Próxima ação sugerida

PLACEHOLDER_NEXT_ACTION

## Cobertura

PLACEHOLDER_COVERAGE_SUMMARY

Detalhes em [[08-meta/coverage]].

## Confiança média

PLACEHOLDER_CONFIDENCE_AVG

Detalhes em [[08-meta/confidence]]. Perguntas abertas em [[08-meta/questions]].

## Como navegar

### Em Obsidian/Logseq (recomendado)

1. Abra a pasta `.first-plan/` como vault no Obsidian
2. Wikilinks `[[secao]]` ficam clickáveis automaticamente
3. Use a vista **Graph** para ver dependências entre seções
4. Use **Backlinks** para descobrir o que referencia cada arquivo

### Em GitHub web

Wikilinks aparecem como texto plano. Para navegar:
- Use a estrutura de pastas (cada seção é uma pasta)
- Os arquivos têm nomes descritivos (`stacks.md`, `errors.md`, etc)

### Via terminal

```bash
# Buscar referências a uma seção
grep -rn "\[\[02-conventions/errors\]\]" .first-plan/

# Listar todos os arquivos
find .first-plan/ -name "*.md" | sort
```

## Como atualizar

- Mudou código? Rode `/first-plan:refresh`. O hook `PostToolUse` ja marcou as seções afetadas como stale em [[08-meta/coverage]].
- Quer recompilar do zero? Apague `.first-plan/cache/` e rode `/first-plan:init` novamente.
