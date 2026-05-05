---
name: first-plan-reconciliation
description: Skill que define Spec-Code Reconciliation - matching contínuo entre artefatos de intenção (docs, specs, JIRA, GitHub issues) e implementação (código, testes, PRs). Use durante Discovery e em /first-plan:check para identificar feature ja implementada, em flight, drifted ou abandonada.
version: 0.1.0
---

# Spec-Code Reconciliation

Núcleo do diferencial do first-plan. Resolve "Claude reimplementa o que ja existe" e "Claude executa spec já feita".

## Statuses possíveis

```
NOT_STARTED  -> intenção existe mas nenhum código relacionado
SPEC_ONLY    -> documentação completa, zero implementação
IN_PROGRESS  -> implementação parcial, branch ativa, ou TODOs visíveis
IMPLEMENTED  -> código completo, com testes
DRIFTED      -> código existe mas divergiu da spec
ABANDONED    -> branch obsoleta + implementação parcial
```

## Coleta de intenções

Fontes ordenadas por prioridade:

1. **Documentação no repo:**
   - `docs/`, `specs/`, `requirements/`, `rfcs/`
   - Arquivos `*.md` com headings tipo "## Feature:", "## Requisito:", "## RFC"
   - README sections com "Roadmap", "Planned features", "TODO"
   - CHANGELOG (entradas pré-release ou "Unreleased")
   - ADRs (Architecture Decision Records)

2. **Issue trackers via MCP:**
   - JIRA via `mcp__jira-mm__*` (issues recentes do projeto, status != Done)
   - GitHub Issues via `mcp__github-work__list_issues`
   - Linear, Asana se MCPs disponíveis

3. **Git artifacts:**
   - Branches com nome `feat/`, `feature/`, `wip/`
   - PR descriptions / templates
   - Commits com tag `[FEAT]` / `feat:`

4. **Comentários no código:**
   - `// TODO: implement <X>`
   - `// FIXME: this is partial`
   - `// PLANNED: <feature>`

## Algoritmo de classificação

Para cada intenção coletada (chamada de "feature candidate"):

### Passo 1 - Extrair termos-chave

Da descrição/título da intenção, extrair:
- Substantivos (entidades, ações)
- Identificadores prováveis (URLs, endpoints, nomes de classes)
- Stack-specific keywords (e.g., "endpoint POST /users" -> grep por POST /users)

### Passo 2 - Buscar evidência no código

Para cada termo-chave:
- Grep símbolos: nomes de função, classe, tipo, route
- Grep arquivos: nomes de arquivo similar
- AST scan se a stack permite: identificar handlers HTTP que respondem ao endpoint da spec
- Grep testes: arquivos de teste mencionando o termo

Coletar evidência:
- Files matched (paths + lines)
- Test files matched
- Open PRs / branches mentioning the feature

### Passo 3 - Aplicar matriz de classificação

```
                       Code presente   Tests presentes   Branch ativa   PR aberto    Verdict
NOT_STARTED            nao              nao               nao            nao         NOT_STARTED
SPEC_ONLY              nao              nao               nao            nao         SPEC_ONLY*
IN_PROGRESS            parcial          parcial/nao       sim            sim/nao     IN_PROGRESS
IMPLEMENTED            completo         sim               nao            nao          IMPLEMENTED
DRIFTED                completo         sim               nao            nao         DRIFTED**
ABANDONED              parcial          nao               obsoleta(>90d) nao         ABANDONED
```

* SPEC_ONLY = mesma evidência que NOT_STARTED, mas a fonte é uma doc completa (ex: ADR aprovado, RFC mergiada). Diferenciar via tamanho/maturidade da spec.

** DRIFTED requer detecção adicional - veja abaixo.

### Passo 4 - Detecção de DRIFT

Para verificar se feature implementada divergiu da spec:

1. Extrair "asserts" da spec (e.g., "endpoint retorna 201", "campo X é obrigatório", "valida email")
2. Verificar cada assert no código:
   - Endpoint retorna mesmo status?
   - Schema de input/output bate?
   - Validações declaradas estão presentes?
3. Se >= 30% dos asserts não batem -> DRIFTED

DRIFT também é detectado quando:
- Spec menciona library X mas código usa Y
- Spec menciona algorithm A mas código usa B
- Comentários no código contradizem spec

### Passo 5 - Confidence por feature

| Sinal | Impacto |
|-------|---------|
| Match de >= 3 termos-chave em arquivo único | +0.3 |
| Test file irmão tem mesmo termo | +0.2 |
| Branch/PR ativo confirma | +0.2 |
| Spec recente (< 90 dias) | +0.1 |
| Spec antiga + código antigo | -0.2 |
| Termos genéricos (e.g., "user", "auth") sem qualificador | -0.3 |
| Match parcial e ambíguo | -0.2 |

Se confidence < 0.7: marcar como "?" no INDEX e adicionar pergunta em `08-meta/questions.md`.

## Output

Para cada feature, criar `09-features/<slug>.md` baseado no template `feature-template.md`. Atualizar `09-features/INDEX.md` com a matriz consolidada.

## Casos especiais

### Phantom features

Feature aparece em backlog/spec como pendente mas evidência mostra IMPLEMENTED. Sinalizar **prominently** em INDEX.md:

```markdown
## Phantom features (alerta!)

Estas features constam como pendentes em alguma fonte mas evidência mostra que ja foram implementadas. **Antes de planejar trabalho relacionado, validar com humano**.

- Feature X (JIRA-123) - implementada em commit abc1234, mas issue ainda Open. Possivelmente esquecida.
```

### Specs duplicadas

Várias docs descrevem a mesma feature. Detectar via overlap de termos-chave > 70%. Reportar como duplicate em questions.md.

### Specs contraditórias

Duas docs descrevem feature similar mas com requisitos conflitantes. Detectar via análise dos asserts. Reportar em questions.md - é decisão humana qual seguir.

## Atualização durante /first-plan:plan

Quando usuário pede uma feature nova via `/first-plan:plan`:
1. Pegar a descrição
2. Aplicar mesmo algoritmo (extract terms -> search -> classify)
3. Se match com feature existente:
   - IMPLEMENTED -> bloquear, perguntar ao usuário
   - IN_PROGRESS -> avisar do branch/PR ativo, sugerir continuar nele
   - SPEC_ONLY / NOT_STARTED -> confirmar que vai virar feature nova
   - DRIFTED -> perguntar se a feature solicitada é justamente reconciliar drift

Resultado vai pra seção "Verificação de duplicidade" do plano.

## Integração com MCPs

Quando MCPs disponíveis:
- `mcp__jira-mm__jira_search` - listar issues do projeto
- `mcp__github-work__list_issues` - listar issues do repo
- `mcp__github-work__list_pull_requests` - PRs ativos
- `mcp__github-work__search_code` - busca avançada de evidência

Sem MCPs, reduzir cobertura para fontes locais (docs/, branches, código).
