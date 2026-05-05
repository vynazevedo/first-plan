---
name: first-plan-reuse-indexing
description: Skill que define como construir o Reuse Index invertido em 03-reuse/. Use durante Discovery para identificar componentes, utils, types, hooks reutilizáveis no projeto e criar o índice "preciso de X -> use Y".
version: 0.1.0
---

# Reuse Indexing

Como criar `.first-plan/03-reuse/` com índice invertido focado em "intenção -> existente".

## Princípio

A pergunta certa nao é "o que existe?" - é **"preciso fazer X, o que ja existe?"**. O índice é organizado por **intenção**, não por estrutura.

## Categorias de intenção (por linguagem/stack)

### Universais

| Categoria | Exemplo |
|-----------|---------|
| Validação | "validar input HTTP" |
| Logging | "registrar evento" |
| Auth | "verificar JWT" |
| HTTP client | "chamar API externa" |
| Persistência | "buscar registro no DB" |
| Conversão / format | "formatar data" |
| Testing helper | "criar fixture" |
| Crypto | "hash de senha" |

### Frontend específicas

| Categoria | Exemplo |
|-----------|---------|
| UI components | "form com validação" |
| Hooks | "fetch com loading state" |
| State management | "store de carrinho" |
| Routing | "redirect autenticado" |
| Forms | "form com validação" |
| Modals / overlays | "dialog de confirmação" |

## Critério de inclusão

Item entra no índice se:
- **Usado em > 1 lugar** OU **claramente público** (export top-level, nome inequívoco)
- **API estável** (nao é refator em curso, nao tem `@deprecated`)
- **Não-trivial** (re-criar custaria > 5 linhas)
- **Identificável** (nome semântico, não `helper.foo()`)

## Algoritmo

### Passo 1 - Coletar candidatos

Por linguagem, identifica símbolos exportados:
- Go: `func` / `type` em pacote `pkg/` ou `internal/util/`
- TS: `export` em arquivos `lib/`, `utils/`, `helpers/`, `components/`, `hooks/`
- PHP: classes em namespaces `App\Services`, `App\Helpers`
- Python: funções/classes em módulos `utils/`, `helpers/`, `services/`
- Rust: `pub` em `src/lib.rs` e módulos públicos

### Passo 2 - Deduplica e classifica por intenção

Agrupa candidatos por categoria. Mapping comum:

| Nome contém | Categoria |
|-------------|-----------|
| `validate`, `valid`, `check` | Validação |
| `log`, `logger`, `audit` | Logging |
| `auth`, `jwt`, `token`, `permission` | Auth |
| `client`, `request`, `fetch`, `api` | HTTP client |
| `repository`, `dao`, `query`, `db` | Persistência |
| `format`, `parse`, `convert`, `to_` | Format |
| `factory`, `builder`, `mock`, `stub` | Testing helper |
| `hash`, `encrypt`, `decrypt`, `sign` | Crypto |
| `Component`, `View`, `Layout` | UI components |
| `use<Capitalized>` (TS/JS) | React hooks |

### Passo 3 - Para cada item, capturar

```yaml
name: <nome do símbolo>
category: <categoria de intenção>
path: <arquivo>:<linha>
signature: <assinatura completa>
purpose: <inferido por nome + comentário se houver>
usages:
  - <path:line de quem usa>
  ... (até 5 ocorrências)
when_to_reuse: <contexto>
when_not_to_reuse: <contexto exclusionário>
```

### Passo 4 - Escrever os arquivos

`03-reuse/INDEX.md`:
- Tabela top-level por categoria
- Linkando aos arquivos de detalhe

`03-reuse/components.md`, `utils.md`, `types.md`, `hooks.md`:
- Detalhe por item com signature, exemplos, usos

`03-reuse/search.json`:
- Versão machine-readable do mesmo índice
- Schema:
```json
{
  "items": [
    {
      "name": "...",
      "category": "...",
      "path": "...",
      "signature": "...",
      "tags": ["...", "..."],
      "usage_count": 5
    }
  ]
}
```

## Heurísticas para purpose / when

Inferir purpose a partir de:
- Comentário JSDoc / docstring / godoc / rustdoc
- Nome do arquivo + nome da função
- Padrão de uso (se chamada N vezes em handlers, é util de handler)

Inferir "when to reuse":
- Categoria + tipo de input/output
- Localização do arquivo (pkg/util/format = formatadores)

Inferir "when NOT to reuse":
- Comentários explicitamente exclusionários
- Substituições recentes (e.g., novo util em commits recentes para uso futuro - não reusar o velho)

## Cuidados

- **Nao inflar o índice.** Helpers triviais (e.g., `func square(x int) int { return x*x }`) não entram.
- **Não reportar items deprecated.** Se há sinal de "use novo X", marcar antigo como NOT recommended.
- **Não inventar usages.** Se nao encontrou call site, listar 0 e reduzir confidence.

## Quando o índice fica grande

Se o projeto tem > 200 items reusáveis:
- Limitar `INDEX.md` ao top 30 por categoria por usage_count
- Detalhe completo nos arquivos por categoria
- `search.json` tem todos
