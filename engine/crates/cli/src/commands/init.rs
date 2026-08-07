use anyhow::{anyhow, Result};
use clap::Args as ClapArgs;
use crossterm::style::Stylize;
use first_plan_core::init::{self, layers, InitOptions};
use first_plan_core::llm::{self, ProviderKind};
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(ClapArgs)]
pub struct Args {
    /// Diretório raiz do projeto (contém ou receberá `.first-plan/`)
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Provider LLM: openai | anthropic | ollama (default: env FIRST_PLAN_LLM_PROVIDER)
    #[arg(long)]
    pub llm: Option<String>,

    /// Modelo específico (default varia por provider)
    #[arg(long)]
    pub model: Option<String>,

    /// Base URL alternativa (útil para Ollama, LM Studio, self-hosted)
    #[arg(long)]
    pub base_url: Option<String>,

    /// Filtra layers geradas por nome canônico (ex: --layer mission/purpose). Repetível.
    #[arg(long = "layer")]
    pub layers: Vec<String>,

    /// Sobrescreve arquivos existentes em `.first-plan/`
    #[arg(long)]
    pub overwrite: bool,

    /// Max tokens por resposta do LLM
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Lista layers disponíveis e sai
    #[arg(long)]
    pub list_layers: bool,

    /// Dry-run: apenas coleta sinais e imprime, sem chamar LLM nem gravar arquivos
    #[arg(long)]
    pub dry_run: bool,

    /// Output JSON estruturado
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct LayersOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    layers: Vec<LayerInfo<'a>>,
}

#[derive(Serialize)]
struct LayerInfo<'a> {
    name: &'a str,
    section: &'a str,
    output_path: &'a str,
}

#[derive(Serialize)]
struct DryRunOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    dry_run: bool,
    signals: &'a first_plan_core::init::signals::ProjectSignals,
    layers_selected: Vec<&'a str>,
}

pub fn run(args: Args) -> Result<()> {
    if args.list_layers {
        return list_layers_command(args.json);
    }

    let signals = first_plan_core::init::signals::collect(&args.root)?;

    if args.dry_run {
        let selected: Vec<&str> = if args.layers.is_empty() {
            layers::all_layers().iter().map(|l| l.name).collect()
        } else {
            args.layers.iter().map(|s| s.as_str()).collect()
        };
        let out = DryRunOutput {
            schema: "first-plan-init-dry-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            dry_run: true,
            signals: &signals,
            layers_selected: selected,
        };
        if args.json {
            serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
            println!();
        } else {
            println!("Sinais coletados de {}:", args.root.display());
            println!(
                "  README: {} chars",
                signals.readme.as_ref().map(|r| r.len()).unwrap_or(0)
            );
            println!("  Manifestos: {}", signals.manifests.len());
            for m in &signals.manifests {
                println!("    - {}", m.path);
            }
            println!(
                "  Stacks detectadas: {}",
                signals.detected_stacks.join(", ")
            );
            if let Some(g) = &signals.git_activity {
                println!("  Git activity: {} commits (90d)", g.total_commits_90d);
            }
            println!();
            println!("Layers que seriam geradas:");
            for name in &out.layers_selected {
                println!("  - {}", name);
            }
        }
        return Ok(());
    }

    let kind = args
        .llm
        .as_deref()
        .map(ProviderKind::from_str)
        .transpose()
        .map_err(|e| anyhow!("provider inválido: {}", e))?;

    let provider = llm::build(kind, args.model.clone(), args.base_url.clone())
        .map_err(|e| anyhow!("build provider: {}", e))?;

    let opts = InitOptions {
        root: args.root.clone(),
        layer_filter: if args.layers.is_empty() {
            None
        } else {
            Some(args.layers.clone())
        },
        overwrite: args.overwrite,
        max_tokens: args.max_tokens,
    };

    let pb = if !args.json {
        crate::tty::spinner(&format!(
            "generating layers via {} ({})",
            provider.name(),
            provider.model()
        ))
    } else {
        indicatif::ProgressBar::hidden()
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let report = rt
        .block_on(init::run_init(provider.as_ref(), opts))
        .map_err(|e| anyhow!("init falhou: {}", e))?;
    pb.finish_and_clear();

    if args.json {
        serde_json::to_writer_pretty(std::io::stdout().lock(), &report)?;
        println!();
    } else if crate::tty::is_tty() {
        crate::tty::header("Init complete");
        crate::tty::kv("provider", &report.provider);
        crate::tty::kv("model", &report.model);
        crate::tty::kv("elapsed", &crate::tty::humanize_ms(report.elapsed_ms));

        if !report.layers_generated.is_empty() {
            crate::tty::section(&format!(
                "Generated layers ({})",
                report.layers_generated.len()
            ));
            let rows: Vec<Vec<String>> = report
                .layers_generated
                .iter()
                .map(|l| {
                    vec![
                        crate::tty::badge(crate::tty::Severity::Ok, "OK"),
                        l.name.clone().bold().to_string(),
                        l.output_path.display().to_string().dim().to_string(),
                        crate::tty::humanize_bytes(l.bytes_written as u64),
                        crate::tty::humanize_ms(l.elapsed_ms),
                    ]
                })
                .collect();
            crate::tty::table(&["status", "layer", "output", "size", "elapsed"], &rows);
        }

        if !report.layers_skipped.is_empty() {
            crate::tty::section(&format!("Skipped ({})", report.layers_skipped.len()));
            let rows: Vec<Vec<String>> = report
                .layers_skipped
                .iter()
                .map(|l| {
                    vec![
                        crate::tty::badge(crate::tty::Severity::Muted, "SKIP"),
                        l.name.clone().bold().to_string(),
                        l.reason.clone().dim().to_string(),
                    ]
                })
                .collect();
            crate::tty::table(&["status", "layer", "reason"], &rows);
        }
    } else {
        println!(
            "init concluído em {}ms (provider={}, model={})",
            report.elapsed_ms, report.provider, report.model
        );
        println!("Layers geradas ({}):", report.layers_generated.len());
        for l in &report.layers_generated {
            println!(
                "  {}  ->  {}  ({} bytes, {}ms)",
                l.name,
                l.output_path.display(),
                l.bytes_written,
                l.elapsed_ms
            );
        }
        if !report.layers_skipped.is_empty() {
            println!("Layers puladas ({}):", report.layers_skipped.len());
            for l in &report.layers_skipped {
                println!("  {}  ({})", l.name, l.reason);
            }
        }
    }
    Ok(())
}

fn list_layers_command(json: bool) -> Result<()> {
    let all = layers::all_layers();
    if json {
        let out = LayersOutput {
            schema: "first-plan-init-layers-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            layers: all
                .iter()
                .map(|l| LayerInfo {
                    name: l.name,
                    section: l.section,
                    output_path: l.output_path,
                })
                .collect(),
        };
        serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
        println!();
    } else {
        println!("Layers disponíveis ({}):", all.len());
        for l in all {
            println!(
                "  {:<24} section={:<12} output={}",
                l.name, l.section, l.output_path
            );
        }
    }
    Ok(())
}
