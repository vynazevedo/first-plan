# Contribuindo para o first-plan

Obrigado por considerar uma contribuição. Este projeto tem manutenção solo mas leva contribuições externas a sério. Bug reports, melhorias em docs, fixes de compatibilidade cross-tool, testes e novas camadas de features são todos bem-vindos.

Este documento é curto de propósito. Em caso de dúvida, abra uma issue primeiro e discutimos antes de você investir tempo.

## Formas de contribuir

- **Bug reports** - abra uma issue descrevendo o que quebrou, reprodução mínima, comportamento esperado, comportamento real, ambiente
- **Compatibilidade cross-tool** - o projeto visa funcionar em Claude Code, GitHub Copilot CLI, Cursor, Codex, Ollama, etc. Se sua ferramenta preferida tropeça em algo que o first-plan gera, por favor reporte e mande PR se possível
- **Docs** - typos, explicações confusas, exemplos faltando, traduções. README existe em EN e PT-BR em `docs/i18n/`
- **Novas camadas de capacidade** - se você tem uma ideia para uma nova camada no IR (veja roadmap no README), abra uma issue para discutir arquitetura antes de codar
- **Adições de skill ou command** - siga o formato de frontmatter documentado abaixo

## Antes de submeter um PR

Por favor rode esses checks localmente. Os comandos exatos que o CI roda estão em `.github/workflows/lint.yml` e `test.yml` - replique com precisão para evitar que o CI pegue o que você deixou passar.

### Engine Rust

```bash
cd engine
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace --release
```

Se seu toolchain Rust local for mais antigo que o stable do CI, rode `rustup update stable` primeiro. Novas versões do clippy adicionam lints que toolchains antigos não pegam.

### Validação de frontmatter

Toda skill, command e agent tem YAML frontmatter que deve passar validação estrita:

```bash
pip install pyyaml
python3 tools/validate_frontmatter.py
```

Armadilha comum: `argument-hint: [--flag N]` sem aspas é parseado como flow-sequence YAML (array). Loaders estritos como GitHub Copilot CLI >= 1.0.65 silenciosamente descartam a skill. Sempre coloque em aspas: `argument-hint: "[--flag N]"`.

### Arquivos JSON

```bash
for f in $(find . -name "*.json" -not -path "./engine/target/*" -not -path "./.git/*"); do
  python3 -c "import json,sys; json.load(open('$f'))" || echo "invalido: $f"
done
```

### Shell hooks

```bash
find hooks -name "*.sh" -exec shellcheck {} +
```

## Formato de PR

- **PRs pequenos preferidos** - mudanças mecânicas single-purpose são mescladas rápido
- **Título do PR** deve descrever o resultado, não a mudança de código. `fix: quote de valores argument-hint YAML para Copilot CLI carregar todas as skills` é melhor que `atualiza 8 arquivos`
- **Body do PR** deve explicar o porquê. Referencie a issue se houver
- **Inclua verificação** - como você testou que a mudança funciona
- **Uma mudança lógica por PR** - separe mudanças não relacionadas em PRs distintos

## Formato de frontmatter skill / command / agent

### commands/*.md

```yaml
---
description: "Uma frase descrevendo o que este comando faz."
argument-hint: "[--flag valor]"      # opcional, DEVE ser string
allowed-tools: [Read, Bash, Edit]    # opcional, DEVE ser lista de strings
---
```

### skills/*/SKILL.md

```yaml
---
name: first-plan-skill-name          # kebab-case com prefixo first-plan-
description: "Uma frase descrevendo quando usar essa skill. Multi-paragrafo OK mas deve ser string unica, então mantenha em uma linha ou wrap em aspas."
version: "0.8.0"
---
```

### agents/*.md

```yaml
---
name: agent-name
description: "Uma frase descrevendo o que o agent faz."
tools: Read, Glob, Grep, Bash        # string CSV (convenção Claude Code) ou lista
model: sonnet                        # opcional
color: purple                        # opcional
---
```

## Estilo de código

- **Sem emojis ou ícones** em docs ou código, a menos que explicitamente solicitado
- **Sem travessão** `-` nem `-`. Use hífen simples `-`
- **Português com acentos corretos** quando escrever em português (não "publicacao", mas "publicação")
- **Mensagens de commit em português** para commits do mantenedor, seguindo conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`)
- **Mensagens de commit em inglês** para contribuições externas também são bem-vindas
- **Sem comentários em código Rust** a menos que o PORQUÊ seja genuinamente não-óbvio. Doc comments (`///`) e module comments (`//!`) são OK

## Reportando bugs

Inclua:

- O que você esperava
- O que aconteceu de fato
- Reprodução mínima (comandos para rodar, arquivos necessários)
- Ambiente: OS, versão do Claude Code (se aplicável), versão do first-plan, versão da engine (`first-plan-engine --version`)
- Se tentou com a última release

Bugs sobre compatibilidade cross-tool são particularmente valiosos e ganham prioridade. Este projeto pretende funcionar com qualquer ferramenta de AI coding, então se sua ferramenta descarta algo que o first-plan produz, isso é um bug real que queremos consertar.

## Discutindo mudanças de arquitetura

Antes de começar trabalho em uma nova camada de capacidade ou refactor grande, abra uma issue com o design sketch. A arquitetura atual está documentada na seção roadmap do README. Camadas de feature planejadas: Quality (v0.8, shipped), Contracts (v0.9), Evolution (v0.10), Runtime (v0.11), Cross-repo (v0.12), Framework pivot (v1.0).

## Código de conduta

Seja direto, seja técnico, respeite outros contribuidores. Sem assédio, sem ataques pessoais. Divergências sobre direção técnica são saudáveis, divergências sobre pessoas não são.

## Licença

Ao contribuir, você concorda que sua contribuição será licenciada sob a licença MIT, mesma do resto do projeto.

## Dúvidas

Abra uma issue com label `question` ou pinge o mantenedor via GitHub. Este é um projeto pequeno, tempo de resposta é medido em dias, não semanas.
