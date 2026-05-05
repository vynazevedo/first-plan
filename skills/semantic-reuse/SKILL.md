---
name: first-plan-semantic-reuse
description: Skill que faz busca semantica via BM25 sobre o indice de simbolos do projeto. Substitui grep do reuse-index quando o engine nativo esta disponivel. Encontra "preciso de validacao de email" mesmo se a funcao se chama validateEmailRFC. Use durante /first-plan:reuse e /first-plan:plan para identificar componentes reusaveis com matching semantico.
version: 0.4.0
---

# Semantic Reuse via BM25

Habilita busca por **intencao** em vez de busca por nome exato. Isso fecha o gap mais comum entre Claude e o codigo: nomes de simbolos divergem da forma como descrevemos intencoes.

Exemplos:
- Query: "validar email" -> encontra `validateEmailRFC`, `is_valid_email`, `EmailValidator`
- Query: "fetch HTTP com retry" -> encontra `httpClientWithBackoff`, `fetchWithRetries`, `RetryableClient`
- Query: "modal de confirmacao" -> encontra `ConfirmDialog`, `useConfirmModal`, `AlertConfirmation`

## Workflow

### Passo 1 - Detectar engine

```bash
ENGINE=""
for c in "${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine" "${HOME}/.local/bin/first-plan-engine" "$(command -v first-plan-engine 2>/dev/null)"; do
  [ -x "$c" ] && ENGINE="$c" && break
done
```

Se ausente, oferecer `first-plan-engine-bootstrap` skill ou cair no fallback markdown.

### Passo 2 - Garantir indice atualizado

```bash
DB=".first-plan/cache/search.db"
# Re-indexar se DB nao existe ou esta stale (>= 24h)
if [ ! -f "$DB" ] || [ $(find "$DB" -mmin +1440 | wc -l) -gt 0 ]; then
  "$ENGINE" index --repo . --db-path "$DB" --output-json /tmp/idx.json
  echo "indexado: $(jq -r .total_symbols /tmp/idx.json) simbolos em $(jq -r .elapsed_ms /tmp/idx.json)ms"
fi
```

### Passo 3 - Query

```bash
"$ENGINE" search \
  --db-path "$DB" \
  --query "<intencao>" \
  --limit 10 \
  --output-json /tmp/search.json
```

Output JSON `first-plan-search-v1`:

```json
{
  "$schema": "first-plan-search-v1",
  "query": "<intencao>",
  "hits": [
    {
      "symbol": {
        "name": "ValidateEmail",
        "kind": "function",
        "language": "go",
        "path": "pkg/validation/email.go",
        "line": 12,
        "signature": "func ValidateEmail(s string) error",
        "doc": "Valida formato RFC 5322"
      },
      "score": 8.42,
      "matched_tokens": ["validate", "email"]
    }
  ]
}
```

### Passo 4 - Renderizar para usuario

Para `/first-plan:reuse <intencao>`, mostrar top 5 com:
- Nome + path:line
- Signature
- Doc resumida
- Score (ajuda o usuario calibrar relevancia)
- Matched tokens (transparencia sobre o ranking)

Para `/first-plan:plan` (verificacao automatica de reuse), filtrar score >= 5.0 (threshold empirico) e listar candidatos no plano.

## Quando usar BM25 vs grep

| Situacao | BM25 (engine) | Grep (fallback) |
|----------|--------------|-----------------|
| Query "preciso de X" abstrato | Vence facil | Falha |
| Query exata por substring | Empata ou vence (ranking) | Funciona |
| Codebase < 100 simbolos | Empata | Funciona |
| Codebase > 1000 simbolos | Vence (rapido + ranqueado) | Lento, lista flat |
| Sem engine disponivel | N/A | Funciona |

## Cuidados

- **Matched tokens transparentes** - sempre listar quais tokens da query bateram. Ajuda usuario detectar quando ranking deu match espurio.
- **Score nao e absoluto** - so faz sentido relativo a outros hits da mesma query. 8.0 numa query pode ser excelente, 8.0 em outra pode ser fraco.
- **Index pode estar stale** - se passou de 24h ou foram feitos muitos commits, sugerir re-indexar antes de query critica.
- **Linguagens suportadas (v0.4.0)**: Go, Rust, TypeScript/JavaScript, Python, PHP. Outras caem no fallback grep.

## Ate onde BM25 chega vs embeddings ML (v0.4.1+)

BM25 acerta quando:
- Query e codigo compartilham vocabulario (validate, email, format, retry, modal)
- Doc strings tem palavras semelhantes a intencao

BM25 falha quando:
- Sinonimos puros sem overlap (query "user" mas codigo so usa "person")
- Conceitos abstratos sem palavras tecnicas comuns

Para o segundo caso, embeddings ML (v0.4.1) acertam, mas v0.4.0 ja resolve 80% dos casos com latencia <50ms e binario de 1MB.
