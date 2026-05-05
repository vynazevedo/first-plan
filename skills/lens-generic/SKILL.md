---
name: first-plan-lens-generic
description: Stack lens fallback para projetos onde nenhuma lens específica detectou. Use quando o projeto tem estrutura inusual, stack rara (Elixir, OCaml, Haskell, Erlang, Zig, etc) ou poliglota além das lenses cobertas. Aplica heurística genérica baseada em extensões e comportamento observável.
version: 0.1.0
---

# Lens Generic

Fallback aplicado quando nenhuma lens específica casa. **Confidence base é menor** (max 0.7) para refletir incerteza.

## Heurística por extensão

| Extensão | Stack provável |
|----------|----------------|
| `.ex` / `.exs` | Elixir |
| `.erl` / `.hrl` | Erlang |
| `.hs` | Haskell |
| `.ml` / `.mli` | OCaml |
| `.zig` | Zig |
| `.nim` | Nim |
| `.cr` | Crystal |
| `.jl` | Julia |
| `.lua` | Lua |
| `.r` / `.R` | R |
| `.scala` / `.sbt` | Scala |
| `.clj` / `.cljs` | Clojure |
| `.fs` / `.fsi` | F# |
| `.kt` (sem build.gradle) | Kotlin script |
| `.dart` (sem pubspec) | Dart standalone |

## Algoritmo

1. **Detectar linguagem dominante** por extensão majoritária no projeto
2. **Identificar manifesto** se houver (Mix.exs, dune-project, stack.yaml, Project.toml, etc)
3. **Inferir framework dominante** por imports/uses comuns na linguagem
4. **Aplicar contrato comum** com confidence reduzida

## Contrato comum (mesmo das lenses específicas)

Mesmo sem conhecer detalhes da linguagem, extrair:

### Estrutura
- Entry point (main file ou bin/)
- Pastas top-level com >5 arquivos
- Padrões de naming de arquivos

### Errors
- Como erros são representados? (excepções, Result, tagged tuples, etc)
- Há padrão consistente?

### Testing
- Há pasta de testes?
- Qual o framework (procurar em deps/manifest)?

### Logging
- Há lib de log identificável?

### DI / composition
- Onde objetos / processos são compostos?

### Observabilidade
- Há instrumentação visível?

## Confidence

- Max 0.7 (vs 0.9 das lenses específicas)
- Min 0.4 quando estrutura é caótica ou identificação fica ambígua

## Quando criar lens específica

Se o usuário rodar Discovery múltiplas vezes em projetos da mesma stack rara, vale criar `lens-<stack>/SKILL.md` específica. Sinais:
- Confidence consistentemente baixa em projetos da mesma stack
- Padrões idiomáticos não capturados
- Findings importantes ficando sub-representados

## Output

Padrão, com nota explícita em cada arquivo do `.first-plan/`:
> Esta seção foi gerada via lens-generic (fallback). Confidence reduzida. Considere criar lens específica para esta stack se o projeto for recorrente.
