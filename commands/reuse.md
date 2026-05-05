---
description: Consulta o Reuse Index invertido. Pergunta "preciso fazer X, o que ja existe?" e recebe lista priorizada com paths e signatures. v0.4.0+ usa BM25 search via engine nativo quando disponivel (semantic matching).
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

### Passo 0 - BM25 engine path (v0.4.0+)

Se `first-plan-engine` esta disponivel (ver skill `first-plan-engine-bootstrap`), preferir busca BM25 sobre o indice de simbolos via skill `first-plan-semantic-reuse`:

```bash
ENGINE=""
for c in "${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine" "${HOME}/.local/bin/first-plan-engine" "$(command -v first-plan-engine 2>/dev/null)"; do
  [ -x "$c" ] && ENGINE="$c" && break
done

if [ -n "$ENGINE" ]; then
  DB=".first-plan/cache/search.db"
  # Re-indexar se ausente ou >= 24h velho
  if [ ! -f "$DB" ] || [ $(find "$DB" -mmin +1440 2>/dev/null | wc -l) -gt 0 ]; then
    "$ENGINE" index --repo . --db-path "$DB" --output-json /tmp/idx.json > /dev/null
  fi

  "$ENGINE" search --db-path "$DB" --query "$ARGUMENTS" --limit 10 --output-json /tmp/search.json
  # Renderizar JSON como tabela markdown
  # Veja skill semantic-reuse para detalhes
fi
```

Se sem engine OU search retornou 0 hits com score >= 3.0, fallback markdown abaixo.

### Passo 1 - Pre-flight (fallback)

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
