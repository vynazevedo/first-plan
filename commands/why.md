---
description: Consulta a camada compilada para responder "por que X existe?". Argumento pode ser símbolo, path, padrão ou decisão. Cruza topology, conventions, rationale, history.
argument-hint: <símbolo|path|padrão>
allowed-tools: [Read, Glob, Grep, Bash]
---

# /first-plan:why

Query da camada compilada: contexto sobre algo do projeto.

## Argumentos

`$ARGUMENTS` - alvo da consulta. Pode ser:
- Path de arquivo: `.first-plan:why src/auth/jwt.go`
- Símbolo: `/first-plan:why ValidateEmail`
- Padrão: `/first-plan:why "wrapping de erros"`
- Decisão: `/first-plan:why "uso de Bull em vez de BullMQ"`

## Workflow

### Passo 1 - Classificar o argumento

- Path de arquivo (existe no filesystem) -> consulta sobre arquivo
- Símbolo (CamelCase, snake_case identificável) -> consulta sobre símbolo
- Frase entre aspas ou texto livre -> consulta livre

### Passo 2 - Buscar evidência na camada

Para path:
- Em `01-topology/architecture.md` - role do arquivo
- Em `02-conventions/*` - exemplos referenciando este path
- Em `03-reuse/*` - se está catalogado como reusável
- Em `05-risks/*` - se está em alguma lista de risco
- Em `06-rationale/why.md` - decisões inferidas
- Em `01-topology/ownership.md` - dono inferido
- Em `01-topology/activity.md` - hot zone? frozen?
- Git: `git log --follow <path>` para histórico

Para símbolo:
- Grep em `03-reuse/*.md` - ja catalogado?
- Grep no projeto: definição + usos
- Em `02-conventions/*` - exemplo de algum padrão?
- Em `06-rationale/do.md` ou `dont.md` - é um padrão?

Para padrão:
- Grep em `02-conventions/*` - existe regra?
- Grep em `06-rationale/*` - existe rationale?
- Grep em `08-meta/questions.md` - é uma pergunta aberta?

Para decisão:
- Em `06-rationale/why.md`
- Git log com mensagens relacionadas
- ADRs em `docs/adr/` se existirem

### Passo 3 - Sintetizar resposta

Estrutura:

```
sobre: <argumento>

Resumo:
<1-3 linhas direto ao ponto>

Contexto detalhado:
- Localização: <path:line se aplicável>
- Papel no projeto: <inferido de architecture.md>
- Padrões aplicáveis: <linkar conventions>
- Quem domina: <de ownership.md>
- Histórico relevante: <commits/PRs relacionados>
- Decisões inferidas: <de why.md>

Riscos / cuidados:
<de risks/*.md se aplicável>

Como reusar / chamar:
<de reuse/INDEX.md se aplicável>

Confidence: <baseado nos findings>
```

### Passo 4 - Quando nao encontrar

Se a camada não tem informação:
```
Sem informação compilada sobre "<argumento>" em .first-plan/.

Sugestões:
- /first-plan:refresh para garantir camada atualizada
- Buscar diretamente: <comando grep sugerido>
- Adicionar como pergunta: editar .first-plan/08-meta/questions.md
```

## Cuidados

- **Não invente informação.** Se a camada não diz, dizer "sem informação".
- **Cite a fonte** (qual arquivo da camada deu a informação) para que o usuário possa verificar.
