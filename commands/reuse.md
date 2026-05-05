---
description: Consulta o Reuse Index invertido. Pergunta "preciso fazer X, o que ja existe?" e recebe lista priorizada com paths e signatures.
argument-hint: <intenção em texto livre>
allowed-tools: [Read, Grep, Bash]
---

# /first-plan:reuse

Query do Reuse Index invertido.

## Argumentos

`$ARGUMENTS` - intenção em texto livre. Exemplos:
- "validação de email"
- "fetch HTTP com retry"
- "logging estruturado"
- "modal de confirmação"

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/03-reuse/INDEX.md` existe.

### Passo 2 - Match por intenção

1. Identificar categoria provável da intenção (validação, http, logging, ui, etc)
2. Ler `03-reuse/INDEX.md` na categoria correspondente
3. Ler `03-reuse/search.json` se intenção atravessa categorias
4. Buscar matches via:
   - Match por keyword na intenção vs `name`/`purpose`/`tags` dos items
   - Categoria exata
   - Categoria adjacente (e.g., "validação" -> também checar "format")

### Passo 3 - Priorizar

Items rankeados por:
1. Match exato de categoria
2. Match de keyword (peso por proximidade)
3. `usage_count` (mais usado = mais provavelmente bem testado)
4. Recência (atualizado recentemente)

### Passo 4 - Apresentar

```
Top items para "<intenção>"

1. <name> em <path:line>
   <signature>
   Propósito: <purpose>
   Usado em: <usage_count> lugares
   Quando reusar: <when_to_reuse>
   Confidence: <score>

2. <name> em <path:line>
   ...

(até 5 items)

Se nenhum casa:
- Considerar criar novo (justificar no plano)
- Verificar adjacências: /first-plan:reuse "<intenção alternativa>"
- Refresh do índice: /first-plan:refresh 03-reuse
```

### Passo 5 - Se vazio

```
Sem matches para "<intenção>" no Reuse Index atual.

Possíveis razões:
- Categoria não existe no projeto (você seria o primeiro)
- Intenção descrita de forma diferente do código

Sugestão:
- Tente termos alternativos: /first-plan:reuse "<sinônimo>"
- Refresh: /first-plan:refresh
- Se realmente não existe, ao planejar, justificar criação do zero no plano
```

## Cuidados

- Não inventar items que não estão no índice.
- Se a query é muito genérica ("fazer coisa"), pedir refinamento.
