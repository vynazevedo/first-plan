//! Definição declarativa das layers geráveis pelo `init --llm`.
//!
//! Cada `LayerSpec` combina caminho de saída (relativo a `.first-plan/`),
//! nome canônico da seção e um prompt específico que direciona o LLM a
//! produzir o conteúdo daquela layer. O conteúdo é escrito como markdown
//! com frontmatter YAML.

pub struct LayerSpec {
    pub name: &'static str,
    pub section: &'static str,
    pub output_path: &'static str,
    pub prompt: &'static str,
}

/// Subset curado de layers para o MVP do `init --llm`. Expansível.
pub const DEFAULT_LAYERS: &[LayerSpec] = &[
    LayerSpec {
        name: "mission/purpose",
        section: "mission",
        output_path: "00-mission/purpose.md",
        prompt: "\
Você é o gerador do layer `00-mission/purpose.md` do first-plan.

Analise os sinais do projeto abaixo e produza uma descrição concisa do propósito. \
Responda estritamente em markdown com as seções:
- O que este projeto faz (1 parágrafo)
- Para quem (target de usuário, 3-5 bullets)
- Que problema resolve (1 parágrafo)
- Estado / maturidade (1 parágrafo, com base em git activity e conteúdo do README)
- Não-objetivos (3-5 bullets do que o projeto explicitamente NÃO faz)

Não invente. Se não houver evidência para uma seção, escreva `TBD` e justifique.
Não inclua frontmatter YAML; ele será adicionado automaticamente.
Não use emojis.",
    },
    LayerSpec {
        name: "topology/stacks",
        section: "topology",
        output_path: "01-topology/stacks.md",
        prompt: "\
Você é o gerador do layer `01-topology/stacks.md`.

Com base nos manifestos e sinais, liste cada stack detectada. Para cada uma:
- Nome canônico (ex: rust, go, python, typescript)
- Manifesto fonte (ex: Cargo.toml)
- Papel provável no projeto (API / CLI / lib / worker / infra / frontend)
- Diretórios provavelmente relacionados

Formato markdown puro com seção `## Detalhamento` contendo subseção por stack. \
Comece com um `## Resumo` de 1 parágrafo. Não invente stacks que não aparecem \
nos sinais. Não inclua frontmatter YAML. Não use emojis.",
    },
    LayerSpec {
        name: "topology/architecture",
        section: "topology",
        output_path: "01-topology/architecture.md",
        prompt: "\
Você é o gerador do layer `01-topology/architecture.md`.

Descreva a arquitetura de alto nível inferida do tree, manifestos e README:
- Componentes principais (com paths)
- Fronteiras / módulos aparentes
- Fluxo de dependência (se inferível: A depende de B)
- Padrões arquiteturais reconhecíveis (monolito, workspace multi-crate, MVC, hexagonal, etc)

Se algum ponto não for inferível dos sinais, escreva `TBD - depende de investigação`. \
Markdown puro, sem frontmatter, sem emojis.",
    },
    LayerSpec {
        name: "conventions/naming",
        section: "conventions",
        output_path: "02-conventions/naming.md",
        prompt: "\
Você é o gerador do layer `02-conventions/naming.md`.

Baseado no tree e trechos de manifesto/README, deduza convenções de nomenclatura:
- Arquivos (ex: snake_case, kebab-case, PascalCase)
- Diretórios
- Símbolos exportados (se houver amostra)
- Branches / tags git (se houver info)

Para cada convenção, aponte evidência específica dos sinais. Marque como `TBD` \
o que não puder ser inferido. Markdown puro, sem frontmatter, sem emojis.",
    },
    LayerSpec {
        name: "conventions/testing",
        section: "conventions",
        output_path: "02-conventions/testing.md",
        prompt: "\
Você é o gerador do layer `02-conventions/testing.md`.

Deduza a estratégia de teste do projeto dos manifestos e tree:
- Framework(s) de teste
- Localização dos testes (co-located, dir separado)
- Categorias (unit, integration, e2e)
- Coverage / mocking aparente

Aponte evidência dos sinais. Marque `TBD` quando não houver info. Markdown puro, \
sem frontmatter, sem emojis.",
    },
    LayerSpec {
        name: "domain/glossary",
        section: "domain",
        output_path: "04-domain/glossary.md",
        prompt: "\
Você é o gerador do layer `04-domain/glossary.md`.

Extraia termos de domínio recorrentes do README, manifestos e nomes de \
arquivos/pastas. Para cada termo:
- Termo
- Definição inferida (1-2 frases)
- Onde aparece (arquivo/seção)

Se não houver termos claros, escreva `Nenhum termo de domínio recorrente foi \
identificado neste snapshot`. Markdown puro, sem frontmatter, sem emojis.",
    },
    LayerSpec {
        name: "risks/fragile",
        section: "risks",
        output_path: "05-risks/fragile.md",
        prompt: "\
Você é o gerador do layer `05-risks/fragile.md`.

Identifique áreas potencialmente frágeis a partir do git activity (arquivos \
mais alterados nos últimos 90 dias frequentemente indicam churn ou instabilidade):
- Top files por churn com hipóteses sobre a causa
- Sinais de dívida técnica visíveis (arquivos gigantes no tree, nomes com \
  'legacy'/'old'/'temp', TODOs no README)

Diferencie hipóteses de fatos. Marque `Hipótese:` quando aplicável. Markdown \
puro, sem frontmatter, sem emojis.",
    },
    LayerSpec {
        name: "state/current",
        section: "state",
        output_path: "07-state/STATE.md",
        prompt: "\
Você é o gerador do layer `07-state/STATE.md`.

Descreva o estado atual do projeto:
- Versão mais recente (se identificável no manifesto)
- Atividade recente (com base nos últimos commits)
- Tendências (aumento/redução de commits, áreas em foco)
- Próximos passos aparentes (se README ou commits recentes indicam)

Seja factual. Use `TBD` para o que não for inferível. Markdown puro, sem \
frontmatter, sem emojis.",
    },
];

/// Retorna todas as specs disponíveis.
pub fn all_layers() -> &'static [LayerSpec] {
    DEFAULT_LAYERS
}

/// Busca uma spec pelo nome canônico (`section/name`).
pub fn find_layer(name: &str) -> Option<&'static LayerSpec> {
    DEFAULT_LAYERS.iter().find(|l| l.name == name)
}
