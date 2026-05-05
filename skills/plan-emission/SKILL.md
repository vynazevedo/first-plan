---
name: first-plan-plan-emission
description: Skill que define como o comando /first-plan:plan emite um plano detalhado para uma feature. Use quando precisar gerar plano que respeite a camada compilada, verifique duplicidade contra 09-features, identifique reuse e siga template em meta-templates/plan.md. Sempre pausa pedindo aprovação ao final.
version: 0.1.0
---

# Plan Emission

Como gerar o plano da Fase 2 que pausa para aprovação humana.

## Pré-requisitos antes de gerar plano

1. `.first-plan/` existe e `STATE.md` indica fase >= `discovered`
2. Stale check: se `08-meta/coverage.md` lista muitos arquivos stale, recomendar `/first-plan:refresh` antes
3. Feature solicitada (texto livre do usuário)

## Workflow

### Passo 1 - Verificar duplicidade

Antes de qualquer coisa, consultar `09-features/INDEX.md`:

- A feature solicitada bate com alguma feature existente lá?
- Se sim, qual o `status` dela?
  - `IMPLEMENTED` -> **Avisar usuário e perguntar:** "Esta feature ja consta como implementada em <evidência>. Você quer A) revalidar a implementação, B) modificar a feature existente, C) seguir mesmo assim (a matriz pode estar errada)?"
  - `IN_PROGRESS` -> Avisar sobre branch/PR ativo. Sugerir continuar nele em vez de começar do zero.
  - `DRIFTED` -> Sinalizar drift e perguntar se a feature solicitada é justamente reconciliar.
  - `SPEC_ONLY` ou `NOT_STARTED` -> Prosseguir.

Se nao bate com nenhuma: prosseguir, mas avisar que isto vai virar feature nova em `09-features/`.

### Passo 2 - Verificar in-flight

Consultar `07-state/in-flight.md`:
- Algum branch ou PR aberto toca arquivos que provavelmente serão modificados pela feature?
- Se sim, listar no plano em "Riscos e ambiguidades" e recomendar coordenação.

### Passo 3 - Identificar reuse aplicável

Consultar `03-reuse/INDEX.md`:
- Por categoria (validação, http, persistência, etc), buscar componentes que se aplicam à feature
- Listar os candidatos no plano com path e justificativa

### Passo 4 - Mapeamento concreto

Para cada arquivo:
- **Criar:** caminho exato + justificativa de criação do zero (se nao tem precedente)
- **Modificar:** caminho + descrição conceitual da mudança + linhas afetadas se possível

Ordenação:
- Tipos / contratos primeiro
- Lógica de domínio
- Camada de persistência
- Handlers / pontos de entrada
- Testes (mas pode ser intercalado se o projeto faz TDD)

### Passo 5 - Aderência aos padrões

Para cada padrão de `02-conventions/*`, mostrar como a feature respeita:
- Naming: arquivos e símbolos seguem convenção <X>
- Errors: erros tratados com <padrão>
- Logging: pontos de log nos eventos <Y>
- Testing: testes em <localização> com <estilo>
- DI: injeção via <mecanismo do projeto>
- Security: validação em <pontos>

### Passo 6 - Riscos e perguntas

Listar:
- Premissas que dependem de findings com confidence baixa (linkar `08-meta/questions.md`)
- Áreas frágeis tocadas (linkar `05-risks/fragile.md`)
- Conflitos potenciais com in-flight
- Dependências externas que precisam ser confirmadas (lib, infra, decisão arquitetural)

### Passo 7 - Critério de "feito"

Lista de itens verificáveis:
- "Endpoint X retorna 200 com corpo Y para input Z"
- "Teste T cobre cenário C"
- "Lint/typecheck passam"
- "Todos os reuse listados foram efetivamente usados"

### Passo 8 - Out of scope

Listar coisas que apareceram durante o planejamento mas estão fora desta feature. **Importante** porque:
- Evita scope creep durante execução
- Documenta para próximas features
- Mostra ao humano oque foi explicitamente deixado de fora

### Passo 9 - Escrever plano

Usar `${CLAUDE_PLUGIN_ROOT}/meta-templates/plan.md` como base. Salvar em:
```
.first-plan/07-state/plans/<slug>.md
```

Onde `<slug>` é gerado a partir do título da feature em kebab-case.

### Passo 10 - Atualizar STATE

`.first-plan/07-state/STATE.md`:
- `phase: awaiting_approval`
- `active_plan: <slug>`

### Passo 11 - Pausar

Apresentar o plano ao usuário e **encerrar a invocação** sem fazer mais nada. O usuário responde com `/first-plan:execute` (aprova) ou texto com mudanças (revisar).

## Anti-padrões em planos

Evitar:
- Planos vagos ("vou criar um service") - ser específico (path + assinatura)
- Listar arquivos sem justificativa
- Esquecer de checar reuse (sempre olhar `03-reuse/`)
- Misturar refator não-pedido com a feature
- Planos sem critério de "feito"
- Planos que assumem premissas com confidence baixa sem mencionar

## Quando o plano é trivial

Se a feature é < 30 linhas em 1 arquivo, ainda assim emitir plano (curto), porque:
- Documenta a decisão
- Permite revisão antes de executar
- Mantém histórico em `reports/`

Não pular o protocolo só porque é "pequeno".
