---
description: Mostra quem domina determinado arquivo ou pasta, baseado em git history. Inclui top contributor, % de autoria, co-autores notáveis.
argument-hint: <path>
allowed-tools: [Read, Bash]
---

# /first-plan:owner

Ownership inferido por path.

## Argumentos

`$ARGUMENTS` - path de arquivo ou pasta.

## Workflow

### Passo 1 - Pre-flight

Verificar git repo + `.first-plan/01-topology/ownership.md`.

### Passo 2 - Match

Procurar `$ARGUMENTS` em `ownership.md`:
- Match exato de path
- Match de prefixo (e.g., consultar `internal/auth/jwt.go` quando `ownership.md` lista `internal/auth/`)

Se nao encontrar match, rerun inline:
```bash
git shortlog -sne -- <path> | head -10
```

### Passo 3 - Apresentar

```
Ownership de <path>

Top contributor: João Silva <joao@example.com>
  - 47% das alterações (28 commits, 1234 linhas modificadas)
  - Última contribuição: ha 3 dias

Co-autores notáveis (>= 20%):
  - Maria Lima  - 31%  (18 commits)
  - Pedro Souza - 22%  (12 commits)

Outros contribuintes:
  - 5 pessoas com < 10% cada

Atividade temporal:
  - Última modificação: <data> (<X dias atrás>)
  - Modificações nos últimos 30 dias: <N>
  - Modificações nos últimos 90 dias: <M>

Implicações:
- Antes de mudança grande, considerar perguntar ao top contributor
- Alta dispersão = conhecimento difuso, possivelmente padrões inconsistentes
- Última modificação > 1 ano + contribuidor original inativo = candidato a dead code
```

### Passo 4 - Casos especiais

Sem ownership claro (todos < 30%):
```
<path> não tem owner claro - 12 pessoas contribuíram, top com apenas 18%.

Implicação: padrões podem estar inconsistentes. Olhar commits recentes mais que histórico geral.
```

Path órfão (top contributor inativo):
```
<path> é potencialmente órfão.
Top contributor (João Silva) não fez commits há 8 meses.

Considerar:
- Validar se este código ainda é usado (via co-change graph: /first-plan:cochange <path>)
- Procurar outro mantenedor antes de mexer
```

## Cuidados

- Comparar com `01-topology/ownership.md` em vez de sempre rodar git pode dar resultado mais rápido mas menos atualizado.
- Se a camada está stale, oferecer refresh inline.
