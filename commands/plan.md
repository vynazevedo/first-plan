---
description: Gera plano detalhado para uma feature usando a camada compilada. Verifica duplicidade, identifica reuse, mapeia arquivos a criar/modificar, lista riscos. Pausa pedindo aprovação ao final - não executa nada.
argument-hint: <descrição da feature em texto livre>
allowed-tools: [Read, Glob, Grep, Bash, Write, Edit, Task]
---

# /first-plan:plan

Gera plano da Fase 2 - sempre pausa pedindo aprovação humana.

## Argumentos

`$ARGUMENTS` - descrição da feature solicitada em texto livre. Exemplos:
- "adicionar endpoint de export CSV para /orders"
- "implementar autenticação por API key além do JWT existente"
- "adicionar rate limiting por IP no gateway"

## Workflow

### Passo 1 - Pre-flight

1. Verificar `.first-plan/` existe e `STATE.md` indica fase >= `discovered`
2. Verificar se já há plano ativo (`07-state/STATE.md` em `awaiting_approval` ou `executing`):
   - Se sim, perguntar: "Plano <slug> está <fase>. Substituir? (sim/não)"
   - Não prosseguir sem confirmação
3. Verificar staleness: se muitos arquivos stale, recomendar `/first-plan:refresh` antes

### Passo 2 - Verificação de duplicidade

Aplicar skill `reconciliation`:

1. Spawnar `reconciliation-auditor` com `feature_query=<argumento>`:

```
Task(
  subagent_type="reconciliation-auditor",
  description="Duplicate check for plan request",
  prompt="Audit feature: '<arg>'. Apply classification. Return status + evidence + recommendation."
)
```

2. Se status = IMPLEMENTED:
   - **PARAR** - perguntar ao usuário:
     ```
     Esta feature parece já implementada (<evidência>).

     Opções:
     A) Cancelar plano (a feature já existe)
     B) Modificar a feature existente em vez de criar nova
     C) Seguir com plano mesmo assim (pode estar errado a matriz)

     Qual? [A/B/C]
     ```
   - Não prosseguir sem decisão.

3. Se status = IN_PROGRESS:
   - Avisar: "Há trabalho em flight em <branch>/<PR>. Sugiro continuar nesse contexto em vez de plano novo."
   - Perguntar se quer prosseguir mesmo assim.

4. Se status = DRIFTED:
   - Perguntar: "A feature solicitada é justamente reconciliar o drift detectado em <feature>?"

5. Se status = SPEC_ONLY ou NOT_STARTED ou nova: **Prosseguir.**

### Passo 3 - Coleta de contexto relevante

Ler trechos relevantes da camada compilada:

- `01-topology/architecture.md` - onde a feature se encaixa
- `01-topology/boundaries.md` - se feature toca contratos
- `02-conventions/*` - todas as convenções aplicáveis
- `03-reuse/INDEX.md` - candidatos a reuse
- `04-domain/*` - entidades e flows relevantes
- `05-risks/*` - riscos em paths que provavelmente serão tocados
- `06-rationale/*` - decisões a respeitar
- `07-state/in-flight.md` - PRs/branches que podem conflitar
- `08-meta/questions.md` - perguntas abertas relevantes

### Passo 4 - Identificar reuse aplicável

Para cada categoria relevante à feature, consultar `03-reuse/INDEX.md` (skill `reuse-indexing`):
- Listar candidatos com path:line
- Marcar quais serão usados
- Justificar caso vá criar do zero

### Passo 5 - Mapear arquivos

Para cada arquivo:
- **Criar:** caminho exato + tipo + justificativa de criação do zero
- **Modificar:** caminho + descrição conceitual + linhas afetadas

Ordenar por dependência (tipos -> domínio -> persistência -> handlers -> testes), mas respeitando estilo do projeto (TDD pode intercalar testes).

### Passo 6 - Aderência aos padrões

Para cada padrão de `02-conventions/*`, mostrar como a feature respeita.

### Passo 7 - Riscos e perguntas

Listar:
- Premissas com confidence baixa (linkar `08-meta/questions.md`)
- Áreas frágeis tocadas (linkar `05-risks/`)
- Conflitos potenciais com in-flight
- Dependências externas precisando confirmação

### Passo 8 - Critério de "feito" + Out of scope

- Done criteria: lista verificável
- Out of scope: coisas que apareceram mas não fazem parte desta feature

### Passo 9 - Escrever plano

1. Determinar slug em kebab-case da descrição
2. Carregar `${CLAUDE_PLUGIN_ROOT}/templates/plan.md.template`
3. Substituir placeholders
4. Salvar em `.first-plan/07-state/plans/<slug>.md`

### Passo 10 - Atualizar STATE

`.first-plan/07-state/STATE.md`:
- `phase: awaiting_approval`
- `active_plan: <slug>`

### Passo 11 - Apresentar e pausar

Mostrar o plano completo ao usuário e:

```
Plano gerado em .first-plan/07-state/plans/<slug>.md

Para aprovar e executar:
/first-plan:execute

Para ajustar, descreva a mudança e o plano será reescrito.

** AGUARDANDO APROVACAO HUMANA - NENHUMA ACAO ALEM DESTE PLANO ATE QUE VOCE APROVE **
```

**FIM da invocação. Não executar mais nada.** Aguardar próxima mensagem do usuário.

## Cuidados criticos

- **Nunca executar nada do plano dentro do plan command.** Apenas escrever o arquivo de plano.
- **Sempre pausar** ao final - usuario tem que dar /first-plan:execute explicitamente.
- Se durante o planning surgir pergunta crítica (premissa não validável), adicionar em `08-meta/questions.md` e mencionar na seção "Riscos e ambiguidades" do plano.
- **Honrar as 7 regras invioláveis** da skill `protocol`:
  1. Reuse first
  2. Verdade do projeto está no projeto
  3. Sem deps novas (justificar se precisar)
  4. Consistência > elegância
  5. Criação do zero é exceção
  6. Acentuação completa em PT
  7. Tipagem forte
