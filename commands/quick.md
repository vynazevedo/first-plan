---
description: Glance do projeto em 30-60 segundos. Stack detection, top simbolos, atividade git, convencoes basicas. Versao leve do init - serve como primeira impressao antes de rodar analise completa.
argument-hint: ""
allowed-tools: [Read, Bash]
---

# /fp:quick

Resumo executivo do projeto em ~30s. Gera `.first-plan/quick/00-glance.md` com 1 página de contexto suficiente para o Claude começar a ajudar imediatamente, sem rodar o init completo (3-8 min).

## Quando usar

- **Primeira vez no projeto** - quer ver valor em <1 minuto antes de investir 5 min no init
- **Sessão nova num projeto que ja tem `.first-plan/`** - quer recap rapido sem re-rodar tudo
- **Demo/screenshot** - mostra capacidade do plugin sem espera
- **CI/CD ou ambiente restrito** - quer contexto rapido sem indices pesados

Para analise completa (reuse index, reconciliation, co-change graph, provenance), rode `/fp:init` depois.

## Workflow

### Passo 1 - Engine check

```bash
ENGINE=$(which first-plan-engine 2>/dev/null || echo "${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine")
if [ ! -x "$ENGINE" ]; then
  echo "Engine nao encontrado. /fp:quick requer first-plan-engine v0.7.0+."
  echo "Instale: cargo install --git https://github.com/vynazevedo/first-plan --path engine/crates/cli"
  echo "Ou baixe binario: https://github.com/vynazevedo/first-plan/releases/latest"
  exit 0
fi
```

### Passo 2 - Rodar glance e salvar arquivo

```bash
mkdir -p .first-plan/quick
"$ENGINE" quick --root "$PWD" --output .first-plan/quick/00-glance.md --json > /tmp/fp-quick.json
```

Tempo tipico: 1-5s (rapido em projetos pequenos, ate 30-40s em monorepos grandes).

### Passo 3 - Ler o JSON pra apresentar resumo

```bash
cat /tmp/fp-quick.json
```

JSON estrutura:

```json
{
  "elapsed_ms": 1289,
  "root": "/path/to/project",
  "stacks": [{"language": "Rust", "manifest": "engine/Cargo.toml"}],
  "entry_points": ["engine/crates/cli/src/main.rs"],
  "top_symbols": [{"name": "Args", "kind": "struct", "file": "...", "line": 17}],
  "git_activity": {
    "recent_commits": ["..."],
    "hot_files": [{"path": "...", "change_count": 12}],
    "active_authors": [{"name": "...", "commit_count": 47}]
  },
  "conventions": {
    "naming": "snake_case predominante (45 de 50 arquivos)",
    "test_frameworks": ["Rust #[cfg(test)] mod tests inline"]
  },
  "commands": ["cargo build --release (in engine)", "cargo test --workspace (in engine)"]
}
```

### Passo 4 - Apresentar resumo ao usuario

Formatar de forma curta:

```
=== /fp:quick concluido em <elapsed>ms ===

Stacks: <linguagens separadas por virgula>
Entry points: <N> detectados
Top simbolos: <N> amostrados
Commits recentes: <N>
Hot files (90d): <N>
Autores ativos: <N>
Naming: <convencao detectada>

Glance salvo em: .first-plan/quick/00-glance.md

Comandos sugeridos pra esse projeto:
  - <comando 1>
  - <comando 2>
  - ...

Proximos passos:
  /fp:init           # analise completa (3-8 min)
  /fp:lsp-status     # detecta LSP servers
```

### Passo 5 - Casos especiais

#### Projeto sem manifests reconhecidos

Se `stacks` está vazio no JSON:

```
=== /fp:quick concluido ===

Nenhum manifest reconhecido encontrado.
Projeto pode ser: Bash/dotfiles, documentacao pura, ou stack nao suportada.

Glance gerado mesmo assim com simbolos e atividade git (se aplicavel).
```

#### Projeto sem git

Se `git_activity` está null no JSON: omitir secao git no resumo, mencionar que projeto nao tem .git.

#### Engine ausente

Cair em fallback simples se necessario:

```bash
echo "Engine nao disponivel - usando fallback minimo"
echo "Stacks (manifest sweep):"
for m in Cargo.toml go.mod package.json pyproject.toml composer.json Gemfile; do
  [ -f "$m" ] && echo "  - $m"
  find . -maxdepth 2 -mindepth 2 -name "$m" 2>/dev/null | sed 's|^|  - |'
done
```

## Diferencas vs /fp:init

| Aspecto | /fp:quick | /fp:init |
|---------|-----------|----------|
| Tempo | 1-5s (tipico) ate 30s (monorepo) | 3-8 min |
| Output | 1 arquivo (~100 linhas markdown) | 10 camadas IR completa |
| Stack detection | Manifest sweep root + 1 nivel | Stack Lens Engine completo |
| Symbol extraction | grep heuristica (top 25) | tree-sitter AST completo (centenas) |
| Patterns | Inferencia naming + test frameworks | Pattern archeologist com confidence scoring |
| Reuse index | nao | sim |
| Reconciliation | nao | sim |
| Co-change graph | nao | sim |
| Provenance | nao | sim |
| Living layer | nao | sim |
| Subagents usados | nenhum | 3 (discovery, pattern, reconciliation) |
| Custo em tokens | minimo (~3-5KB output) | medio-alto |
| Quando usar | primeira impressao, demo, sessoes curtas | producao continuada |

## Cuidados

- `/fp:quick` **nao substitui** o init - apenas oferece overview rapido
- Heuristicas podem errar com confidence baixa (e por isso e rapido)
- Nao usa subagents - tudo via engine + Claude lendo JSON
- Se ja existe `.first-plan/` completo, este comando cria `.first-plan/quick/` em paralelo (nao sobrescreve outras camadas)
- Output e propositalmente conciso para caber no contexto sem sobrecarregar
- Top simbolos sao **amostra** (heuristica grep), nao listagem completa - use `/fp:init` para extracao tree-sitter precisa
