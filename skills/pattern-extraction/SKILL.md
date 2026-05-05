---
name: first-plan-pattern-extraction
description: Skill que define como extrair padrões do código com confidence scoring. Use durante Discovery quando precisar identificar convenções, anti-padrões ou padrões idiomáticos do projeto. Cada padrão extraído deve vir com exemplo concreto do código (path:line).
version: 0.1.0
---

# Pattern Extraction

Como transformar observações do código em "padrões" registráveis em `.first-plan/02-conventions/`, `06-rationale/do.md`, `06-rationale/dont.md`.

## Princípio

**Sem exemplo do código, sem padrão.** Toda regra extraída precisa de pelo menos 1 exemplo concreto (`path:line`). Sem exemplo, vira pergunta em `08-meta/questions.md` ou é descartada.

## Algoritmo de extração

### 1. Amostragem

Para cada categoria (errors, naming, testing, etc), amostre arquivos representativos:
- Top 10 arquivos por linhas de código (não-teste)
- Top 5 arquivos modificados nos últimos 30 dias
- 3 arquivos de teste de cada pasta principal

Limite: máx 50 arquivos lidos por categoria. Se ficar muito ruidoso, abortar com confidence baixa.

### 2. Identificação de padrão candidato

Em cada arquivo amostrado, busque:
- Repetição estrutural (5+ ocorrências da mesma forma)
- Imports comuns (mesmo símbolo importado em N+ arquivos)
- Receitas (e.g., "todo handler começa com `func Handle...(ctx Context, req Request) (Response, error)`")
- Comentários proibitivos (`// don't use X`, `// use Y instead`)
- Código deletado em commits recentes (sinal de anti-padrão)

### 3. Confidence scoring

Cada padrão candidato ganha confidence baseado em sinais convergentes:

| Sinal | Peso |
|-------|------|
| 5+ ocorrências consistentes | +0.3 |
| 10+ ocorrências consistentes | +0.5 |
| Mesmo padrão em testes e código de produção | +0.2 |
| Comentário no codebase explicitamente preferindo o padrão | +0.2 |
| Documentado em README/CLAUDE.md/docs/ | +0.2 |
| Ocorre em código recente (últimos 90 dias) | +0.1 |
| Não tem variantes inconsistentes | +0.2 |
| Tem variantes inconsistentes (alguns fazem A, outros B) | -0.3 |
| Apenas 1-2 ocorrências | -0.4 |
| Apenas em código antigo (>1 ano sem update) | -0.2 |
| Conflita com outro padrão extraído | -0.3 |

Soma final clamped em [0.0, 1.0]. Threshold default 0.7.

### 4. Anti-padrões (o que o projeto evita)

Detectar via:
- Ausência consistente de algo comum em outros projetos similares (e.g., zero `any` em projeto TS estrito = sinal forte de "evitar `any`")
- Comentários `// don't`, `// avoid`, `// nao usar`, `// removed because`
- Commits/PRs com mensagens "remove", "deprecate", "migrate away from"
- Padrão substituído por outro (e.g., todos os `console.log` foram trocados por `logger.info` em commits recentes)

Confidence para anti-padrão tem regra extra: requerer **pelo menos 2 sinais distintos** para considerar válido.

### 5. Exemplificação obrigatória

Cada padrão extraído gera entrada no formato:

```markdown
### Padrão: <nome>

- **Resumo:** <1 linha>
- **Por que existe (inferido):** <opcional, se houver evidência>
- **Exemplo:**
  ```<lang>
  <trecho real do código>
  ```
  Visto em: `<path>:<line>`
- **Quando aplicar:** <contexto>
- **Confidence:** <score>
```

Sem o exemplo, **descartar o padrão ou rebaixar a pergunta**.

## Categorias e onde escrever

| Categoria | Arquivo de saída |
|-----------|------------------|
| Naming | `02-conventions/naming.md` |
| Errors | `02-conventions/errors.md` |
| Testing | `02-conventions/testing.md` |
| Logging | `02-conventions/logging.md` |
| DI | `02-conventions/di.md` |
| Security | `02-conventions/security.md` |
| Padrões positivos cross-cutting | `06-rationale/do.md` |
| Anti-padrões | `06-rationale/dont.md` |
| Decisões inferidas | `06-rationale/why.md` |

## Quando virar pergunta em vez de padrão

Se confidence < 0.7 mas o tópico é importante (e.g., naming convention é crítico):
1. Não escrever o padrão como verdade
2. Adicionar entrada em `08-meta/questions.md` com hipóteses A/B/C
3. Anotar em `02-conventions/<category>.md` que "convenção indeterminada - ver questions.md"

## Cuidados

- **Não inventar regras genéricas.** Se o projeto não tem padrão claro, o output é "indeterminado", não "use snake_case porque é a convenção da linguagem".
- **Não impor best practices externas.** Se o projeto usa `var` em vez de `let` em todo lugar, o padrão extraído é "use `var`", não "deveria usar `let`".
- **Variantes inconsistentes** são sinal de migração em curso ou de equipes desalinhadas. Reportar ambas, com nota de "variante minoritária possivelmente em migração".
