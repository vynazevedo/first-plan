---
name: first-plan-lens-engine
description: Skill que define o motor de detecção e roteamento de stack lenses do first-plan. Mapeia manifestos (go.mod, package.json, etc) -> stack -> papel -> lens skill correspondente. Use quando precisar decidir qual lens aplicar a uma pasta durante Discovery, ou quando estender suporte a uma nova stack.
version: 0.1.0
---

# Lens Engine

Motor que detecta stacks por manifesto, infere papel da pasta e roteia para a skill `lens-<stack>` correspondente.

## Tabela de detecção (manifesto -> stack -> papel)

| Manifesto | Stack | Pistas adicionais para inferir papel |
|-----------|-------|--------------------------------------|
| `go.mod` | Go | `cmd/` = CLI/binário; `internal/` = lib privada; `pkg/` = lib pública; presença de `chi`/`gin`/`echo`/`fiber` = HTTP API; `kafka-go`/`amqp`/`segmentio/kafka-go` = consumer/producer |
| `package.json` | Node/TS | `next.config.*` = Next.js; `vite.config.*` = SPA Vite; `nest-cli.json` = NestJS; `astro.config.*` = Astro; `remix.config.*` = Remix; `sveltekit` em deps = SvelteKit; `@nuxt/*` = Nuxt; sem framework + `bin/` = CLI Node |
| `composer.json` | PHP | `artisan` = Laravel; `bin/console` = Symfony; `index.php` em pasta `public/` sem framework = legacy |
| `pyproject.toml` / `setup.py` / `requirements.txt` | Python | `manage.py` = Django; `fastapi` em deps = FastAPI; `flask` em deps = Flask; `litestar` = Litestar; `celery` em deps = worker; `streamlit` = data app |
| `Cargo.toml` | Rust | `[[bin]]` = binário; `[lib]` = lib; presença de `actix-web`/`axum`/`rocket` = HTTP API |
| `*.tf` | Terraform | `main.tf` na raiz = root módulo; `modules/` = library; presença de `provider "aws"` etc para identificar plataforma |
| `Gemfile` | Ruby | `config.ru` = Rack; `bin/rails` = Rails; presença de `sidekiq` em Gemfile = worker |
| `pubspec.yaml` | Flutter / Dart | `lib/main.dart` = app Flutter; `bin/` = CLI Dart |
| `Package.swift` | Swift | `Sources/` = lib/app Swift |
| `build.gradle.kts` ou `build.gradle` | Kotlin/Java/Android | `AndroidManifest.xml` = Android app; `application` plugin = backend |
| `*.csproj` ou `*.sln` | .NET (C#) | `<OutputType>Exe</>` = CLI/web; `Microsoft.AspNetCore` = ASP.NET |
| `mix.exs` | Elixir | `phoenix` em deps = Phoenix |
| `deno.json` ou `deno.jsonc` | Deno | inferir papel pelo entry script |
| `bun.lockb` ou `bunfig.toml` | Bun | similar a Node/TS, framework por inspeção |

## Algoritmo de detecção

```
1. Para cada pasta candidata (root + subdiretórios não-ignorados):
   a. Listar arquivos top-level
   b. Match contra tabela acima (primeiro match ganha; se múltiplos, todos contam = polyglot)
   c. Se match: registrar stack + papel inferido
   d. Se nenhum match: aplicar lens-generic
   
2. Excluir pastas:
   - node_modules, vendor, target, dist, build, .next, .nuxt, venv, .venv
   - .cache, coverage
   - Pastas listadas em .gitignore

3. Limites:
   - Profundidade máxima: 4 níveis
   - Tamanho máximo por arquivo de manifesto inspecionado: 1MB
   - Em monorepo grande (>50 manifestos), aplicar amostragem por pasta principal
```

## Roteamento para lens-<stack>

Após detectar stack X, carregar e aplicar `skills/lens-<x>/SKILL.md`. Cada lens segue o **contrato comum**:

### Contrato comum de lens

Cada `lens-<stack>/SKILL.md` deve providenciar:

1. **Detecção fina** - como diferenciar variantes da stack (Next vs Remix em Node; Django vs FastAPI em Python)
2. **Extração de padrões obrigatórios:**
   - Layout de pastas (entry points, organização)
   - Convenção de errors
   - Convenção de logging
   - Convenção de testes
   - Convenção de DI / composição
   - Padrões de validação (se aplicável)
   - Code generation (se aplicável)
3. **Output esperado** - quais arquivos do `.first-plan/` cada lens preenche
4. **Confidence rules** - quando aumentar/diminuir confidence

## Stack desconhecida / fallback

Se detectado manifesto não-listado ou nenhum manifesto:
1. Aplicar `lens-generic/SKILL.md`
2. Registrar em `.first-plan/01-topology/stacks.md` como "stack: unknown, lens: generic"
3. Reduzir confidence base para 0.5 (max 0.7)

## Como estender (adicionar nova stack)

1. Criar `skills/lens-<nova-stack>/SKILL.md` seguindo o contrato comum
2. Adicionar entrada na tabela acima (atualizar este arquivo)
3. Não há outra mudança necessária - o engine descobre a skill via filesystem

## Polyglot (múltiplas stacks na mesma pasta)

Quando uma pasta tem múltiplos manifestos (ex: `package.json` + `composer.json` em projeto Laravel + frontend):
- Registrar todas as stacks
- Aplicar cada lens em paralelo
- Nas seções compartilhadas (architecture.md, boundaries.md), reconciliar findings

## Output em `.first-plan/01-topology/stacks.md`

Cada stack detectada vira uma seção no `stacks.md` com:
- Manifesto (caminho)
- Versão (do manifesto)
- Papel inferido
- Lens aplicada
- Pastas relacionadas
- Framework dominante
- Confidence
