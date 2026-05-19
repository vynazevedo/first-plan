---
description: Mostra quais Language Servers (LSP) estao instalados, quais o projeto atual precisa, e comandos de instalacao para os faltantes.
argument-hint: ""
allowed-tools: [Read, Bash]
---

# /first-plan:lsp-status

Inspeciona Language Server Protocol no ambiente atual. Reporta servers detectados em PATH, versoes, cobertura no projeto e sugestoes de instalacao.

## Workflow

### Passo 1 - Engine check

Verificar se `first-plan-engine` esta disponivel:

```bash
ENGINE=$(which first-plan-engine 2>/dev/null || echo "${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine")
if [ ! -x "$ENGINE" ]; then
  echo "Engine nao encontrado. /first-plan:lsp-status requer engine binario >= v0.6.0"
  exit 0
fi
```

### Passo 2 - Status

```bash
"$ENGINE" lsp status --root "$PWD" --json > /tmp/lsp-status.json
```

JSON retornado:

```json
{
  "engine_version": "0.6.0",
  "project_root": "/path/to/project",
  "project_needs": ["rust-analyzer", "typescript-language-server"],
  "servers": [
    {
      "id": "RustAnalyzer",
      "name": "rust-analyzer",
      "language": "Rust",
      "binary": "rust-analyzer",
      "installed": true,
      "path": "/home/user/.cargo/bin/rust-analyzer",
      "version": "rust-analyzer 0.3.2127",
      "install_cmd": "rustup component add rust-analyzer"
    }
  ]
}
```

### Passo 3 - Apresentar

```
=== LSP servers (8 suportados) ===

Status:
  [OK]   rust-analyzer                Rust                    (path: ~/.cargo/bin/rust-analyzer)
  [--]   gopls                        Go                      install: go install golang.org/x/tools/gopls@latest
  [--]   pyright                      Python                  install: npm install -g pyright
  [OK]   typescript-language-server   TypeScript/JavaScript   (path: /usr/local/bin/typescript-language-server)
  [--]   intelephense                 PHP                     install: npm install -g intelephense
  [OK]   clangd                       C/C++                   (path: /usr/bin/clangd)
  [--]   ruby-lsp                     Ruby                    install: gem install ruby-lsp
  [--]   lua-language-server          Lua                     install: brew install lua-language-server

Projeto atual:
  Stacks detectadas via manifests: Rust (Cargo.toml), TypeScript (package.json)
  rust-analyzer:               OK (cobertura 100%)
  typescript-language-server:  OK (cobertura 100%)

Recomendacoes:
  Nenhuma - todos os servers necessarios estao instalados.
```

### Passo 4 - Faltantes no projeto

Se existem stacks detectadas sem server instalado:

```
=== LSP servers ===

Projeto atual:
  Stacks: Go (go.mod), Python (pyproject.toml)
  gopls:    nao instalado -- fallback tree-sitter ativo
  pyright:  nao instalado -- fallback tree-sitter ativo

Sugestao de instalacao:

  # Go
  go install golang.org/x/tools/gopls@latest

  # Python
  npm install -g pyright

Sem LSP, operacoes seguem funcionando via tree-sitter + grep
(precisao ~70% vs 100% com LSP).
```

### Passo 5 - Engine ausente

Se engine binario nao existe:

```
Engine nativo nao detectado. Instale com:

  cargo install --git https://github.com/vynazevedo/first-plan --path engine/crates/cli

ou baixe binario pre-compilado:

  https://github.com/vynazevedo/first-plan/releases/latest

Sem engine, plugin continua funcionando 100% via fallback markdown.
LSP integration requer engine v0.6.0+.
```

## Cuidados

- Status reporta apenas o que esta em PATH no momento; mudancas exigem reload de shell
- Versoes nao sao validadas contra minimos requeridos (futuro v0.6.1)
- Skill `lsp-aware` documenta detalhes das operacoes; este comando so reporta estado
