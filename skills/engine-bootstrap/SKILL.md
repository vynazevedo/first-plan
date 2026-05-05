---
name: first-plan-engine-bootstrap
description: Skill que detecta e instala o binário nativo first-plan-engine (Rust) na primeira invocação. Use quando algum command (cochange, hash, futuramente ast/index/search) for invocado e o binário não estiver presente. Oferece download via GitHub Releases. Plugin funciona sem o engine (graceful fallback), mas com ele fica 10-100x mais rápido.
version: 0.3.0
---

# Engine Bootstrap

Skill que gerencia o lifecycle do binário nativo `first-plan-engine`.

## Quando ativar

Qualquer command que se beneficia do engine (`/first-plan:cochange`, `/first-plan:refresh` quando recalcula co-change, `/first-plan:hot` em monorepo grande, etc) deve consultar esta skill antes de cair no fallback markdown.

## Detecção

Locais onde procurar o binário, na ordem:

```
1. ${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine
2. ${HOME}/.local/bin/first-plan-engine
3. $(which first-plan-engine)  # se está no PATH
```

Verificar via:

```bash
ENGINE=""
for candidate in \
  "${CLAUDE_PLUGIN_ROOT}/engine/bin/first-plan-engine" \
  "${HOME}/.local/bin/first-plan-engine" \
  "$(command -v first-plan-engine 2>/dev/null)"; do
  if [ -x "$candidate" ]; then
    ENGINE="$candidate"
    break
  fi
done
```

## Instalação

Se ausente e o command que precisou foi invocado, oferecer:

```
O command /first-plan:<X> ficaria 10-100x mais rapido com o engine nativo (Rust binary).
Quer baixar agora? (~5MB, single-file static binary, opt-in)

Opcoes:
A) Sim - download automatico para ${CLAUDE_PLUGIN_ROOT}/engine/bin/
B) Nao - usar fallback markdown (continua funcionando)
C) Manual - instrucoes para download via cargo install ou wget
```

### Download automatico (opcao A)

Detectar arch + OS, baixar release matching:

```bash
ARCH=$(uname -m)        # x86_64 ou aarch64
OS=$(uname -s)          # Linux ou Darwin

case "$OS-$ARCH" in
  Linux-x86_64)   TARGET="x86_64-unknown-linux-musl" ;;
  Linux-aarch64)  TARGET="aarch64-unknown-linux-musl" ;;
  Darwin-x86_64)  TARGET="x86_64-apple-darwin" ;;
  Darwin-arm64)   TARGET="aarch64-apple-darwin" ;;
  *)              echo "unsupported: $OS-$ARCH"; exit 1 ;;
esac

DEST="${CLAUDE_PLUGIN_ROOT}/engine/bin"
mkdir -p "$DEST"

URL="https://github.com/vynazevedo/first-plan/releases/latest/download/first-plan-engine-${TARGET}.tar.gz"
curl -fsSL "$URL" | tar xz -C "$DEST"
chmod +x "$DEST/first-plan-engine"

"$DEST/first-plan-engine" --version
```

### Build from source (opcao C)

```bash
git clone https://github.com/vynazevedo/first-plan
cd first-plan/engine
cargo install --path crates/cli
```

Binário fica em `~/.cargo/bin/first-plan-engine`.

### Manual download

Releases em https://github.com/vynazevedo/first-plan/releases.
Download binário matching arch, extract, chmod +x, place em PATH.

## Versão check

Após detectar binário, comparar versão com a esperada pelo plugin:

```bash
INSTALLED=$("$ENGINE" --version | awk '{print $2}')
EXPECTED="0.3.0"
# se diferentes, sugerir update
```

## Fallback graceful

Se usuário recusa download OU download falha (rede restrita, corp proxy):
- Continuar com comportamento markdown atual
- Logar one-liner: "engine ausente, usando fallback markdown"
- NÃO abortar o command

## Output esperado

Esta skill retorna ao caller:
- `engine_path: <path>` se disponível
- `engine_path: null` se ausente

Caller decide se usa engine ou fallback baseado nesse retorno.

## Cuidados

- **Nunca instalar sem confirmação** do usuário
- **Nunca falhar o command** por causa do engine ausente - sempre fallback
- **Verificar checksum** do release seria ideal mas v0.3.0 inicial fica sem (TODO v0.4.0)
- Em corp/restricted envs, instrução de download manual deve estar visível
