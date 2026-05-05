---
description: Mostra riscos conhecidos para um path - código frágil, sem testes, magic, debt. Use antes de mexer em arquivos críticos.
argument-hint: <path>
allowed-tools: [Read, Grep, Bash]
---

# /first-plan:risk

Riscos catalogados em `.first-plan/05-risks/` para um path específico.

## Argumentos

`$ARGUMENTS` - path de arquivo ou pasta.

## Workflow

### Passo 1 - Pre-flight

Verificar `.first-plan/05-risks/*` existe.

### Passo 2 - Buscar matches

Para cada arquivo em `05-risks/`:
- `fragile.md` - listas de items frágeis
- `untested.md` - items sem testes
- `magic.md` - código clever
- `debt.md` - TODOs/FIXMEs

Match por path:
- Exato: caminho idêntico
- Prefixo: o path solicitado é prefixo de algum item listado
- Regex pattern (se item é wildcard tipo `internal/legacy/*`)

Também consultar:
- `01-topology/activity.md` - hot zone implica volatilidade
- `01-topology/ownership.md` - sem dono claro = risco
- `09-features/INDEX.md` - feature em DRIFTED associada ao path

### Passo 3 - Apresentar

```
Riscos para <path>

Frágil: <Sim/Não>
<detalhe se sim>

Sem testes: <%>
<detalhe>

Magic / clever code: <Sim/Não>
<exemplos se sim>

Débito catalogado:
- TODOs: <count>
- FIXMEs: <count>
- HACKs: <count>

Sinais adicionais:
- Hot zone? <Sim/Não> (X commits em 30 dias)
- Owner: <inferido ou "no clear owner">
- Features afetadas: <linkar 09-features se aplicável>

Recomendação:
<síntese baseada nos sinais>
```

### Passo 4 - Sem riscos catalogados

```
Nenhum risco específico catalogado para <path>.

Isso pode significar:
- Path realmente seguro
- Discovery não cobriu este path (ver 08-meta/coverage.md)

Para garantir: /first-plan:refresh
```
