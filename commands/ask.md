---
description: Lista perguntas abertas em 08-meta/questions.md que precisam de resposta humana antes da camada considerar findings com confidence completa.
argument-hint: [--answer Q<n> "resposta"]
allowed-tools: [Read, Edit, Glob]
---

# /first-plan:ask

Lista e gerencia perguntas abertas para o humano.

## Argumentos

`$ARGUMENTS`:
- vazio: lista perguntas abertas
- `--answer Q<n> "resposta"`: registra resposta para pergunta específica

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/08-meta/questions.md` existe.

### Passo 2 - Lista (default)

Ler arquivo, mostrar perguntas com `answered: false`:

```
Perguntas abertas (<N>)

Q1 [naming]
   Arquivos usam camelCase em alguns lugares e snake_case em outros. Qual é a convenção atual?
   Sinal observado: internal/userHandler.go vs internal/payment_service.go
   Hipóteses:
     A) Migração em curso de camelCase -> snake_case
     B) Inconsistência histórica sem padrão claro
     C) Convenção é por tipo de arquivo
   Impacto: afeta nomeação de novos arquivos

Q2 [errors]
   ...

Para responder uma pergunta:
/first-plan:ask --answer Q1 "snake_case é o padrão atual, camelCase é legacy a migrar"
```

### Passo 3 - Mode --answer

1. Ler `08-meta/questions.md`
2. Localizar `Q<n>`
3. Atualizar:
   - `answered: true`
   - `answer: "<texto da resposta>"`
   - `answered_at: <timestamp>`
4. Atualizar contadores no frontmatter (`open_questions`, `answered_questions`)
5. Confirmar: "Q<n> respondida. Considere /first-plan:refresh para aplicar a resposta nas seções afetadas."

### Passo 4 - Aplicar respostas em refresh

A resposta não modifica automaticamente as seções do `.first-plan/`. O usuário deve rodar `/first-plan:refresh` em seguida, e os subagents vão considerar as respostas em `questions.md` quando reanalisar.

## Cuidados

- Não inventar respostas se o usuário pediu a lista vazia ("não há perguntas abertas").
- Não modificar status de pergunta sem confirmação.
- Histórico de perguntas respondidas vai pra seção "Perguntas respondidas" do mesmo arquivo, mantida em ordem cronológica reversa.
