---
name: first-plan-protocol
description: Esta skill DEVE ser carregada em qualquer comando do plugin first-plan. Define o protocolo PLAN-FIRST (Discovery -> Plan -> Approval -> Execution -> Report), as 7 regras invioláveis e o comportamento esperado em cada fase. Ative quando o usuário invocar /first-plan:* ou mencionar "first-plan", "context compilation", "discovery layer", "spec-code reconciliation".
version: 0.1.0
---

# Protocol - first-plan

Skill base obrigatória do plugin. Toda execução de comando `/first-plan:*` carrega esta skill.

## Filosofia

**Antes de escrever uma linha de código, entenda o projeto.** O plugin existe pra forçar essa disciplina. Não improvisar, não impor padrões externos, não duplicar trabalho.

## Protocolo PLAN-FIRST (obrigatório)

```
Discovery -> Plan -> Approval -> Execution -> Report
```

Fases bloqueantes. Cada uma só inicia quando a anterior estiver concluída e validada.

### Fase 1 - Discovery

Disparada por `/first-plan:init`. Resultado: `.first-plan/` populado.

Mapear:
- Topologia (stacks, arquitetura, boundaries, deployment)
- Convenções (naming, errors, testing, logging, di, security)
- Reuse Index invertido (componentes, utils, types, hooks)
- Domínio (glossário, entidades, flows)
- Riscos (fragile, untested, magic, debt)
- Rationale (do, dont, why)
- Spec-Code Reconciliation (matriz feature × status × evidência)
- Git intelligence (activity, ownership, in-flight)

Cada finding tem `confidence: 0.0-1.0`. Findings com `confidence < 0.7` viram entradas em `.first-plan/08-meta/questions.md`.

### Fase 2 - Plan

Disparada por `/first-plan:plan <feature>`. Resultado: `.first-plan/07-state/plans/<slug>.md`.

Antes de escrever o plano:
1. Consultar `.first-plan/09-features/` - **a feature ja existe?**
2. Consultar `.first-plan/07-state/in-flight.md` - **alguém ja esta fazendo?**
3. Consultar `.first-plan/03-reuse/INDEX.md` - **o que reusar?**

O plano deve seguir o template em `${CLAUDE_PLUGIN_ROOT}/meta-templates/plan.md`. **Pausa pedindo aprovação humana** ao final.

### Fase 3 - Approval (gate humano)

Estado: `awaiting_approval` em `.first-plan/07-state/STATE.md`.

Aguardar:
- `/first-plan:execute` → prossegue
- Mensagem com mudança → reescreve plano e volta a aguardar

**Não executa nada antes da aprovação.** Não há atalho.

### Fase 4 - Execution

Disparada por `/first-plan:execute`. Resultado: código modificado conforme plano + atualização do STATE.

Durante:
- Seguir o plano à risca, na ordem definida
- Commits pequenos e atômicos no padrão do projeto
- Se descobrir algo que invalide o plano (premissa errada, contrato diferente, erro de discovery): **PARAR e reportar**, não improvisar
- Atualizar STATE.md a cada passo

### Fase 5 - Report

Após execução completa. Resultado: `.first-plan/07-state/reports/<slug>.md`.

Incluir:
- O que foi feito
- O que foi reusado vs criado do zero (com justificativa)
- Desvios do plano (se houver)
- Riscos remanescentes
- Sugestões fora do escopo

Recomendar `/first-plan:refresh` para atualizar a camada.

## 7 Regras invioláveis

Estas regras nascem do briefing original do projeto. Não negociar.

### 1. Reuse first

Antes de criar qualquer função, componente, hook, helper ou tipo, **verificar `.first-plan/03-reuse/INDEX.md`**. Se existir equivalente, usar. Criar do zero é exceção que exige justificativa explícita no plano.

### 2. A verdade do projeto está no projeto

Não impor "best practices" externas. Se o projeto faz feio mas consistente, seguir o feio. Refatoração não está no escopo a menos que solicitada.

### 3. Sem dependências novas

Usar apenas o que ja está em `go.mod` / `package.json` / `composer.json` / etc. Adicionar lib exige justificativa explícita no plano e aprovação separada.

### 4. Consistência > elegância

Se o projeto tem padrão estabelecido, seguir mesmo que voce ache que existe forma melhor. Sugestões de melhoria vão pra "fora do escopo" do report, nunca executadas no fluxo da feature.

### 5. Criação do zero é exceção

Permitida apenas quando não há precedente (primeiro componente de um tipo no projeto, primeiro endpoint de um domínio). Sempre justificar no plano + report.

### 6. Acentuação completa em português

Comentários, mensagens de erro, docs, commits - tudo com acentuação correta.

### 7. Tipagem forte

Respeitar o nível de tipagem do projeto. Nada de `any` / `interface{}` / `dynamic` se o projeto é estrito.

## Comportamento "não inventa, pergunta"

Quando confidence < 0.7 em qualquer finding crítico para uma decisão:
1. Não assumir
2. Adicionar entrada em `08-meta/questions.md` com hipóteses
3. Em `/first-plan:plan`, listar as perguntas pendentes na seção "Riscos e ambiguidades" do plano
4. Aguardar resposta humana antes de prosseguir com partes que dependem da pergunta

## Estado entre sessões

Tudo persiste em arquivos:
- `.first-plan/07-state/STATE.md` - máquina de estado, `phase` atual
- `.first-plan/07-state/plans/` - planos ativos
- `.first-plan/08-meta/cache.json` - hashes de arquivos para invalidação

Ao reabrir o Claude Code, o entry point é `STATE.md`. Claude lê `phase` e age conforme.

## Detecção de invalidação durante execução

Se durante `/first-plan:execute` Claude perceber que:
- Um arquivo que o plano assumia existir não existe
- Um símbolo que o plano referenciava tem assinatura diferente
- Um teste que deveria passar mostra comportamento inesperado
- Discovery indicou algo que se mostra incorreto

Então: **parar imediatamente, atualizar STATE.md para `paused`, reportar ao usuário com:**
1. Premissa que falhou
2. Evidência da falha
3. Opções: replanejar / abortar / continuar com adaptação
