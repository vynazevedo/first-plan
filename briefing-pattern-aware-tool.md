# Briefing: Pattern-Aware Feature Development Tool

## Objetivo
Construir uma ferramenta (skill / plugin / agente) para Claude Code capaz de operar em projetos multi-stack desconhecidos — qualquer combinação de stacks (Go, Node/TS, PHP, Python, Rust, Terraform, mobile, etc.), em monorepo ou poly-repo — e implementar features novas com **aderência absoluta aos padrões existentes do projeto**. A ferramenta deve se comportar como um dev sênior recém-chegado: lê tudo antes de escrever uma linha, copia o estilo da casa e não inventa moda.

## Modo de operação obrigatório: PLAN-FIRST
Antes de qualquer escrita de código, a ferramenta produz um plano detalhado e **aguarda aprovação explícita**. Nada de "vou implementando enquanto exploro". Fluxo estrito:

```
Discovery → Plan → Approval → Execution → Report
```

Se durante a execução algo invalidar o plano, parar e reportar. Sem improviso.

---

## Fase 1 — Discovery (análise profunda do projeto)

Para cada stack/pasta envolvida, mapear:

### Estrutura geral
- Layout de diretórios e princípio de organização (feature-based, layer-based, DDD, etc.)
- Pontos de entrada (`main.go`, `cmd/`, `app/`, `pages/`, `src/`)
- Configuração: `.env`, config files, feature flags
- Build, lint, format, hooks de pre-commit

### Padrões de código (transversais)
- Convenções de nomenclatura (arquivos, funções, tipos, variáveis)
- Estilo arquitetural (handlers/services/repositories, hexagonal, clean, MVC, etc.)
- Tratamento de erros (sentinel, wrapping, tipos customizados)
- Injeção de dependência
- Logging, observabilidade, instrumentação
- Convenções de teste (estrutura, mocks, fixtures, table-driven)

### Detecção de stacks (pré-requisito)
Antes de qualquer mergulho, identificar:
- Quais stacks estão presentes (Go, Node/TS, PHP, Python, Rust, Terraform, mobile, etc.)
- Papel de cada pasta (API, worker, CLI, frontend web, mobile, infra, lib compartilhada, gateway, ETL...)
- Manifesto de cada uma (`go.mod`, `package.json`, `composer.json`, `pyproject.toml`, `Cargo.toml`, `*.tf`, `Gemfile`, `pubspec.yaml`)
- Topologia (monorepo? poly-repo agrupado? quem chama quem? quem provisiona o quê?)

### Lenses específicas por stack
Aplicar o conjunto relevante de lenses conforme o que foi detectado. Cada lens é uma checklist do que olhar nessa stack:

**Go**
- Layout (`cmd/`, `internal/`, `pkg/`)
- Error handling (sentinel, wrap, custom types)
- Uso de `context.Context`
- Concorrência (goroutines, channels, errgroup)
- Code generation (`go generate`, mocks, sqlc, oapi-codegen)
- Build tags e compilation constraints

**Node / TypeScript**
- Workspace manager (pnpm, yarn, npm, turbo, nx)
- Module system (ESM/CJS) e tsconfig (strictness, paths)
- Async patterns (Promises, async/await, RxJS)
- Runtime (Node version, Bun, Deno) e bundler (esbuild, swc, webpack, vite)
- Validação de schemas (Zod, Yup, io-ts, class-validator)

**Frontend (React, Next.js, Vue, Svelte, etc.)**
- Sistema de design e biblioteca de componentes existente
- Gerenciamento de estado (Context, Zustand, Redux, React Query, etc.)
- Data fetching (server components, route handlers, SWR, wrapper de axios)
- Estilização (Tailwind, CSS Modules, styled-components)
- Roteamento (app router, pages router, file conventions)
- Loading/error/empty states
- Formulários e validação (RHF, Zod, etc.)

**PHP**
- Composer e autoload (PSR-4)
- Framework (Laravel, Symfony, Slim, raw) e suas convenções
- Service container e DI
- Migrations e ORM (Eloquent, Doctrine)
- PSR compliance (PSR-12, PSR-7, PSR-15)
- Padrões de fila (Horizon, Messenger)

**Python**
- Gerenciador de dependências (poetry, uv, pip, pipenv)
- Framework (FastAPI, Django, Flask, Litestar)
- Packaging (`src/` layout vs flat) e namespace
- Async (asyncio, anyio, trio)
- Validação (Pydantic, marshmallow, attrs)
- Tipagem (mypy, pyright, strictness level)

**Terraform / IaC**
- Estrutura de módulos (root, módulos reutilizáveis, hierarquia)
- Backend de state (S3+DynamoDB, Terraform Cloud, etc.) e locking
- Estratégia de environments (workspaces, dirs separados, tfvars por env)
- Versionamento de providers e module pinning
- Naming, tagging e convenções de recursos
- Outputs e data sources entre módulos

**Mobile (React Native, Swift, Kotlin, Flutter)**
- Navegação (Expo Router, React Navigation, Navigation Component, etc.)
- Estado e persistência local
- Módulos nativos e bridges
- Build e distribuição (EAS, Fastlane, Xcode/Gradle configs)

**Outras stacks (Rust, Elixir, Ruby, Java, .NET, etc.)**
Para qualquer stack não listada explicitamente, derivar a lens dinamicamente: identificar manifesto, framework dominante, convenções de organização, padrões idiomáticos da comunidade — e aplicar a mesma estrutura mental (layout / erros / DI / persistência / testes / observabilidade).

### Cross-stack (contratos e fluxos entre fronteiras)
- Contratos de API entre serviços (HTTP, gRPC, GraphQL, tRPC, OpenAPI, AsyncAPI)
- Schemas compartilhados e geração de tipos (protobuf, OpenAPI codegen, packages internos)
- Mensageria e eventos (payloads de fila, schemas de evento, contratos pub/sub)
- Outputs de infra consumidos por aplicações (Terraform → env vars, ARNs, secrets, connection strings)
- Fluxo de autenticação ponta-a-ponta (quem emite, quem valida, propagação de claims)
- Observabilidade correlacionada (trace context, correlation ID, propagação entre stacks)
- Convenções de paginação, filtragem, ordenação consistentes entre APIs
- Internacionalização (fonte da verdade dos textos, sincronização entre back e front)

---

## Fase 2 — Plan (output estruturado)

A ferramenta deve emitir:

1. **Resumo de descoberta** — bullets curtos, sem enrolação, sobre o que foi mapeado.
2. **Mapeamento da feature solicitada para os padrões detectados:**
   - Arquivos novos a serem criados (com caminho exato)
   - Arquivos existentes a serem modificados (com diff conceitual)
   - Componentes / utils / hooks / handlers / tipos existentes que serão **reutilizados**
   - O que precisará ser **criado do zero**, com justificativa de ausência de precedente
3. **Ordem de implementação** (back-first, front-first ou paralelo) e dependências entre passos.
4. **Riscos e ambiguidades** — perguntas diretas pro dev antes de prosseguir.

---

## Fase 3 — Execução

Permitida apenas após aprovação do plano. Durante:

- Seguir o plano à risca.
- Se descobrir algo que o invalide, **parar e reportar** — não improvisar.
- Não introduzir bibliotecas, padrões ou abstrações ausentes do projeto.
- Commits pequenos e atômicos, mensagens no padrão do projeto (Conventional Commits, gitmoji, etc. — o que já existir).

---

## Regras invioláveis

1. **Reuso primeiro.** Antes de criar qualquer função, componente, hook, helper ou tipo, verificar se já existe equivalente. Se existir, usar.
2. **A verdade do projeto está no projeto.** Nada de impor "best practices" externas. Se o projeto faz feio mas consistente, seguir o feio.
3. **Sem dependências novas.** Usar apenas o que já está em `go.mod` / `package.json`. Adicionar lib exige justificativa explícita no plano e aprovação.
4. **Consistência > elegância.** Refatoração não está no escopo a menos que solicitada.
5. **Criação do zero é exceção.** Permitida apenas quando não há precedente (primeiro componente de um tipo, primeiro endpoint de um domínio). Sempre justificar no plano.
6. **Acentuação completa em qualquer texto em português** — comentários, mensagens de erro, docs, commits.
7. **Tipagem forte.** Respeitar o nível de tipagem do projeto. Nada de `any` / `interface{}` se o projeto é estrito.

---

## Entrada esperada do dev

- Caminhos das pastas/repos envolvidos
- Descrição da feature em linguagem natural
- Constraints adicionais (não tocar em X, prazo, módulos congelados, etc.)

## Saída esperada da ferramenta

- **Plano** estruturado conforme Fase 2
- Após aprovação: código PR-ready dividido por arquivo, com diff claro entre modificado e criado
- **Relatório final**: o que foi reusado, o que foi criado do zero (e por quê), riscos remanescentes, sugestões de follow-up fora do escopo

---

## Capacidades técnicas requeridas da ferramenta

- Indexação rápida de múltiplas raízes de projeto (suporte a monorepo e poly-repo)
- Detecção automática de stack por pasta (heurística sobre arquivos de manifesto)
- AST-aware analysis (não só grep) por linguagem — Go via `go/ast`, TS/JS via `ts-morph`, PHP via `nikic/php-parser`, Python via `ast`/`libcst`, Terraform via HCL parser (`hashicorp/hcl`), Rust via `syn`, etc. Fallback grep+regex aceitável para stacks sem parser disponível.
- Cache de descoberta entre execuções (invalida por hash de arquivos relevantes)
- Sessão persistente de contexto entre Discovery → Plan → Execution (sem precisar reanalizar a cada turno)
- Protocolo de aprovação claro (gate humano entre fases)
