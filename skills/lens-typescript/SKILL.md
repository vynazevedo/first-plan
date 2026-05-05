---
name: first-plan-lens-typescript
description: Stack lens para Node/TypeScript. Use durante Discovery quando package.json for detectado. Cobre Next.js, NestJS, Vite SPA, Express, Astro, Remix, Nuxt, plain Node, monorepos pnpm/turbo/nx.
version: 0.1.0
---

# Lens TypeScript / Node

## Detecção fina

Pelo `package.json` + arquivos auxiliares:

| Sinal | Variante |
|-------|----------|
| `next.config.{js,ts,mjs}` | Next.js |
| `nest-cli.json` ou `@nestjs/*` em deps | NestJS |
| `vite.config.{js,ts}` sem framework | SPA Vite |
| `vite.config.{js,ts}` + `@sveltejs/kit` | SvelteKit |
| `astro.config.{js,ts,mjs}` | Astro |
| `remix.config.{js,ts}` | Remix |
| `@nuxt/kit` ou `nuxt.config.{js,ts}` | Nuxt |
| `expo` em deps | Expo (mobile) |
| `react-native` em deps | React Native bare |
| `electron` em deps | Electron |
| `bin` field no package.json sem framework | CLI Node |
| `express` ou `fastify` em deps | Backend tradicional |
| `pnpm-workspace.yaml` | Monorepo pnpm |
| `turbo.json` | Turborepo |
| `nx.json` | Nx |

## Extração de padrões

### Module system e tooling

- `"type": "module"` no package.json -> ESM
- `tsconfig.json` -> verificar `strict`, `noImplicitAny`, `paths`, `target`, `module`
- Bundler: tsx / tsc / esbuild / swc / vite / webpack / rollup
- Runtime: Node, Bun, Deno

### Validação de schemas

- `zod` em deps -> Zod
- `yup` em deps -> Yup
- `io-ts` -> io-ts
- `class-validator` + `class-transformer` -> nestjs-style
- `valibot`, `arktype` -> opções modernas
- `joi` -> Joi (legacy)

Verificar onde validação acontece (boundary handlers? service layer?)

### Async patterns

- Promises com `.then`/`.catch` ou `async/await`?
- `RxJS` em deps -> reactive style (comum em Angular/NestJS)
- `Effect` library?

### Error handling

- Try/catch tradicional
- `Result<T, E>` library (`neverthrow`, `effect`)
- HTTP errors customizados (extends Error)
- Sentry / NewRelic / observability vendor em deps

### Testing

- `vitest`, `jest`, `mocha`, `ava`, `tap`?
- Test runner config (`vitest.config.ts`, `jest.config.js`)
- Coverage tool (c8, nyc)
- E2E (`playwright`, `cypress`, `webdriverio`)?
- Mocks: `vi.mock`, `jest.mock`, `sinon`, `nock` (HTTP)?

### Frontend específico (se Next/Remix/Vite/etc)

- Sistema de design (Tailwind / CSS Modules / styled-components / emotion / vanilla-extract)
- State (`zustand`, `redux`, `jotai`, `recoil`, `pinia`, `mobx`, Context apenas)
- Data fetching (`@tanstack/react-query`, `swr`, `apollo`, `urql`, server components)
- Forms (`react-hook-form`, `formik`, `superforms`)
- Roteamento (App Router vs Pages Router em Next; file-based em Astro)
- i18n (`next-intl`, `i18next`)

### NestJS específico

- DI via decorators (`@Injectable`, `@Inject`)
- Modules / providers / controllers
- Guards / interceptors / pipes
- TypeORM / Prisma / Mongoose

### Logging

- `pino`, `winston`, `bunyan`?
- `console.log` direto (smell)
- Padrão de campos estruturados

## Output em `.first-plan/`

Igual lens-go (`stacks`, `architecture`, `boundaries`, `errors`, `testing`, `di`, `logging`, `reuse`).

Adicional para frontend:
- `03-reuse/components.md` - componentes UI (priorizar Server Components, hooks compartilhados, layout components)
- `03-reuse/hooks.md` - custom hooks reusáveis

## Confidence rules

Aumentar:
- `tsconfig.strict: true` + projeto sem `any` -> alta confiança no padrão "evitar any"
- ESLint config explícita + arquivos em conformidade

Reduzir:
- Mistura de TypeScript + JavaScript no mesmo projeto
- Várias versões de framework (e.g., partes em Next 12, partes em Next 14)
- Testes desatualizados em relação ao código

## Anti-padrões comuns

- `any` em projeto strict
- `// @ts-ignore` ou `// @ts-expect-error` sem comentário explicando
- `console.log` em produção
- `process.env.X` sem schema (smell - deveria ter validação central)
- Promises não-awaited (potential silent failure)
- `useEffect` com lista de deps incorreta (React)
- Componentes "client-only" desnecessariamente em projeto Next App Router
