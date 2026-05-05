---
description: Executa o plano aprovado em .first-plan/07-state/plans/. Segue o plano à risca, para se algo invalidar premissa, gera relatório ao final.
argument-hint: [--dry-run]
allowed-tools: [Read, Glob, Grep, Bash, Write, Edit, MultiEdit, Task]
---

# /first-plan:execute

Fase 3 - executa plano aprovado.

## Argumentos

`$ARGUMENTS`:
- vazio: executa o plano ativo
- `--dry-run`: simula sem aplicar mudanças, mostra diff conceitual

## Workflow

### Passo 1 - Pre-flight

1. Ler `.first-plan/07-state/STATE.md`
2. Verificar `phase: awaiting_approval` E `active_plan` definido
   - Se phase != awaiting_approval: rejeitar com explicação
   - Se sem active_plan: rejeitar
3. Ler `.first-plan/07-state/plans/<active_plan>.md`
4. Ler `.first-plan/08-meta/questions.md`:
   - Se há perguntas marcadas como críticas para este plano e ainda `answered: false`: **PARAR** e listar

### Passo 2 - Atualizar STATE

`.first-plan/07-state/STATE.md`:
- `phase: executing`
- registrar timestamp de início

### Passo 3 - Confirmar premissas

Para cada arquivo que o plano marca como "modificar" ou "reusar":
- Verificar que o path existe
- Verificar que assinatura/símbolo referenciado bate

Se alguma premissa não bate:
- **PARAR**
- `phase: paused`
- Reportar ao usuário:
  ```
  Premissa do plano falhou:
  <descrição>

  Evidência:
  <output do check>

  Opções:
  A) Replanejar: /first-plan:plan <feature> (refaz o plano com camada atualizada)
  B) Adaptar: descreva a alteração e prossiga
  C) Abortar: /first-plan:status (volta para discovered)
  ```
- **Aguardar instrução humana.** Não improvisar.

### Passo 4 - Executar passos na ordem

Para cada passo do plano:

1. Mostrar ao usuário: "Passo X/N: <descrição>"
2. Se `--dry-run`: mostrar diff conceitual, não aplicar
3. Senão: aplicar via Edit/Write/MultiEdit
4. Após aplicar:
   - Verificar que aplicou conforme descrito
   - Se houve linter/formatter automático no projeto, rodar inline (sem corrigir além do plano)
5. Atualizar `STATE.md` com último passo executado

### Passo 5 - Detecção de invalidação durante execução

A cada passo, verificar:
- Símbolo que outro passo do plano vai usar ainda existe?
- Tipos referenciados ainda batem?
- Testes que deveriam continuar passando ainda passam (se rápidos)?

Se algo invalidar plano:
- **PARAR imediatamente**
- `phase: paused`
- Reportar:
  ```
  Plano invalidado durante execução no passo X.
  Razão: <descrição>
  Evidência: <output>

  Estado atual:
  - Passos completos: 1..X-1
  - Passo atual: X (NAO aplicado)
  - Passos restantes: X+1..N (pendentes)

  Opções:
  A) Replanejar a partir daqui
  B) Adaptar manualmente este passo
  C) Reverter o que foi feito (git checkout)
  ```
- **Não improvisar.** Aguardar instrução.

### Passo 6 - Após sucesso, gerar report

Quando todos os passos completam:
1. Spawnar geração do report usando `templates/report.md.template`
2. Salvar em `.first-plan/07-state/reports/<slug>.md`
3. Atualizar `09-features/<slug>.md` se a feature ja estava lá (mudar status para IMPLEMENTED) ou criar nova entry

### Passo 7 - Atualizar STATE

`.first-plan/07-state/STATE.md`:
- `phase: done`
- `last_completed_plan: <slug>`
- `active_plan: null`

### Passo 8 - Recomendar refresh

```
Feature implementada com sucesso.

Resumo:
- Arquivos criados: <count>
- Arquivos modificados: <count>
- Reuse aplicado: <count> items
- Criação do zero (justificado): <count>
- Testes adicionados: <count>

Report completo: .first-plan/07-state/reports/<slug>.md

Próxima ação:
- Refresh recomendado pra atualizar a camada com o novo código:
  /first-plan:refresh

- Para próxima feature: /first-plan:plan <descrição>
```

## --dry-run

Simula execução sem aplicar:
- Mostra arquivo por arquivo o diff conceitual
- Mostra ordem
- Identifica premissas que falhariam
- Não modifica nada (incluindo STATE.md)
- Útil para revisar uma vez mais antes de aprovar de fato

## Padrão de commit

Se o projeto usa commits:
- Commits pequenos e atômicos
- Mensagens no padrão detectado em `02-conventions/naming.md` (Conventional Commits ou outro)
- **NÃO comitar automaticamente** a menos que o plano explicitamente pediu - default é deixar mudanças no working tree pro usuário revisar

## Cuidados invioláveis

1. **Nunca exceder o plano.** Se descobrir bug ou debt durante execução, adicionar em `report` na seção "Sugestões fora do escopo" - não corrigir no fluxo.
2. **Nunca skip teste/lint.** Se o projeto roda lint pre-commit, deixar rodar.
3. **Nunca usar deps novas** que não foram aprovadas no plano.
4. **Sempre parar e perguntar** quando algo invalida premissa.
5. **Sempre gerar report** ao final, mesmo que execução foi parcial.
