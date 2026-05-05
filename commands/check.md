---
description: Verifica se uma feature específica já existe (em parte ou inteira) no projeto. Antes de planejar, use isto para evitar reimplementação.
argument-hint: <descrição da feature>
allowed-tools: [Read, Glob, Grep, Bash, Task]
---

# /first-plan:check

Verificação ad-hoc de duplicidade para uma feature antes de plan.

## Argumentos

`$ARGUMENTS` - descrição da feature em texto livre. Exemplos:
- "endpoint de export CSV de pedidos"
- "autenticação via JWT"
- "rate limiting no gateway"

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/` existe.

### Passo 2 - Match contra matriz existente

Aplicar algoritmo da skill `reconciliation`:
1. Extrair termos-chave da descrição
2. Match contra entries em `09-features/INDEX.md` por overlap de termos

Se match encontrado: mostrar feature existente + status:

```
Match encontrado: F5 - "Two-Factor Authentication" (status: IN_PROGRESS)

Detalhes:
- Fonte: JIRA-789
- Confidence: 0.78
- Evidência: branch feat/2fa, PR #42 aberto há 3 dias
- Última atualização: 2026-04-30

Recomendação:
- Esta feature está em progresso. Considerar continuar nesse PR em vez de começar do zero.
- Para ver detalhes: cat .first-plan/09-features/two-factor-authentication.md
- Para colaborar: gh pr checkout 42
```

### Passo 3 - Sem match na matriz - busca direta

Se não match na matriz (pode ser feature nova ou matrix stale):

Spawnar `reconciliation-auditor` com `feature_query=<descrição>`:

```
Task(
  subagent_type="reconciliation-auditor",
  description="Ad-hoc feature check",
  prompt="Audit single feature: '<argumento>'. Apply full classification algorithm against current codebase. Return status + evidence."
)
```

### Passo 4 - Reportar resultado

Caso 1 - feature realmente nova:
```
"<descrição>" parece ser feature nova - sem evidência de implementação ou spec.

Status sugerido se planejar: NOT_STARTED
Próximo passo: /first-plan:plan "<descrição>"
```

Caso 2 - feature existente que não estava na matriz (matrix stale):
```
Encontrei evidência da feature mas não estava em 09-features/.

Evidência:
- internal/handler/export.go:45 (handler já existe)
- internal/handler/export_test.go (com tests)

Status inferido: IMPLEMENTED (confidence 0.78)

Sugestão:
- Não duplicar - feature parece já existir
- Atualize a matriz: /first-plan:refresh 09-features
- Se quer modificar a feature existente, especifique melhor o que deseja
```

Caso 3 - feature parcial detectada:
```
Implementação parcial detectada para "<descrição>".

Evidência parcial:
- internal/export/csv.go:1-30 (função básica)
- Sem tests
- Branch feat/csv-export com 5 commits

Status inferido: IN_PROGRESS (confidence 0.65)

Sugestão:
- Continuar trabalho existente em vez de começar do zero
- Ver progresso: git log feat/csv-export
```

## Cuidados

- Se a descrição é muito genérica, pedir refinamento antes de classificar.
- Se confidence da classificação ad-hoc < 0.6, marcar como "duvidoso" e sugerir refresh + revisão.
