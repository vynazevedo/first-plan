---
name: first-plan-git-intelligence
description: Skill que extrai inteligência do git history - heatmap de atividade, ownership, branches/PRs in-flight. Use durante Discovery e em /first-plan:hot, /first-plan:owner, /first-plan:in-flight. Tudo via comandos git read-only.
version: 0.1.0
---

# Git Intelligence

Extração de sinal do git que humanos veem mas IA tipicamente ignora.

## Pré-requisito

Projeto deve ser git repo. Se nao for, todas as seções correlatas em `.first-plan/01-topology/activity.md`, `01-topology/ownership.md`, `07-state/in-flight.md` ficam vazias com nota explicativa.

```bash
test -d .git || echo "Not a git repo"
```

## Activity heatmap

### Top arquivos por commits recentes

```bash
git log --since="90 days ago" --name-only --pretty=format: \
  | grep -v '^$' \
  | sort | uniq -c | sort -rn \
  | head -20
```

Output em `.first-plan/01-topology/activity.md` na seção "Top 20 arquivos".

### Top pastas

```bash
git log --since="90 days ago" --name-only --pretty=format: \
  | grep -v '^$' \
  | xargs -I{} dirname {} \
  | sort | uniq -c | sort -rn \
  | head -20
```

### Hot zones

Definição: pastas/arquivos com >= 10 commits em 30 dias.

```bash
git log --since="30 days ago" --name-only --pretty=format: \
  | grep -v '^$' \
  | sort | uniq -c \
  | awk '$1 >= 10' \
  | sort -rn
```

### Frozen zones

Pastas sem commits ha > 180 dias.

```bash
# Para cada arquivo do projeto:
git log -1 --format="%ad %P" --date=format:"%Y-%m-%d" -- <file>
# Filtrar onde data < (hoje - 180d)
```

(Aproximação rápida via comparação de timestamps.)

## Ownership

### Top contributor por path

```bash
git shortlog -sne --all -- <path>
```

Pegar o autor com maior contagem para cada pasta principal. Output em `01-topology/ownership.md`.

### Co-autores notáveis

Autores com >= 20% das alterações naquele path. Útil para identificar pares que dominam juntos.

### Arquivos sem dono claro

Arquivos modificados por > 10 pessoas sem nenhum com >= 30% das alterações. Marcar em ownership.md como "no clear owner".

### Arquivos órfãos

Última modificação > 1 ano AND contribuidor original nao apareceu nos últimos 6 meses. Marcar como "orphan" em ownership.md - candidatos a dead code.

## In-flight (branches + PRs)

### Branches locais e remotas

```bash
git for-each-ref --sort=-committerdate \
  --format='%(refname:short) %(committerdate:relative) %(authorname)' \
  refs/heads/ refs/remotes/
```

Para cada branch que nao é main/master:
- Diff vs main: `git rev-list --count <branch>...origin/main`
- Idade do último commit
- Status inferido:
  - active = commit < 7 dias
  - stale = commit > 7 dias e < 30 dias
  - abandoned = commit > 30 dias

### Pull Requests abertos

Se MCP `github-work` disponível:
```
mcp__github-work__list_pull_requests com state=open
```

Senão, se `gh` CLI instalado:
```bash
gh pr list --state open --json number,title,author,branch,createdAt,updatedAt
```

Senão, deixar nota em `in-flight.md` explicando que PR data nao disponível.

### WIP detection

Branches/PRs com sinais explícitos de WIP:
- Nome de branch contém `wip`, `draft`, `temp`, `experiment`
- PR title contém `[WIP]`, `[DRAFT]`, `Draft:`
- PR no estado draft (mais formal)

Listar em `07-state/in-flight.md` na seção "WIP detectado".

### Conflitos potenciais

Se branch X modificou arquivos A,B,C e há feature pedida que provavelmente toca esses arquivos:
- Marcar como "conflito potencial" no plano
- Recomendar coordenação ou rebase

## Outputs

### `01-topology/activity.md`
- Top 20 arquivos por commits recentes
- Top 20 pastas
- Hot zones
- Frozen zones
- Velocity por área principal

### `01-topology/ownership.md`
- Por pasta principal: top contributor + %
- Co-autores notáveis
- Arquivos sem dono claro
- Arquivos órfãos

### `07-state/in-flight.md`
- Branches locais
- Branches remotas
- PRs abertos
- WIP detectado
- Conflitos potenciais

## Performance

Em monorepos grandes, `git log` pode ser lento. Mitigações:
- Limitar window: `--since="90 days ago"`
- Usar `--diff-filter` para ignorar deletes
- Cachear resultado em `08-meta/cache.json` com TTL de 24h

## Privacy

Os comandos acima leem dados locais do git. Não exfiltram nada. Se MCP github-work for usado, só leitura também. Não escrever nada externamente.
