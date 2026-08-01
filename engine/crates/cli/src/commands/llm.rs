use anyhow::{anyhow, Result};
use clap::{Args as ClapArgs, Subcommand};
use first_plan_core::llm::{self, ChatMessage, ProviderKind};
use serde::Serialize;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub op: Op,
}

#[derive(Subcommand)]
pub enum Op {
    /// Envia uma mensagem ao provider configurado e imprime a resposta.
    Chat(ChatArgs),
    /// Lista providers suportados.
    Providers(ProvidersArgs),
}

#[derive(ClapArgs)]
pub struct ChatArgs {
    /// Provider: openai | anthropic | ollama (default: env FIRST_PLAN_LLM_PROVIDER ou openai)
    #[arg(long)]
    pub provider: Option<String>,

    /// Modelo a usar (default varia por provider)
    #[arg(long)]
    pub model: Option<String>,

    /// Base URL alternativa (útil para Ollama, LM Studio, self-hosted)
    #[arg(long)]
    pub base_url: Option<String>,

    /// System prompt opcional
    #[arg(long)]
    pub system: Option<String>,

    /// Prompt inline (mutuamente exclusivo com --prompt-file e --stdin)
    #[arg(long)]
    pub prompt: Option<String>,

    /// Lê prompt de arquivo
    #[arg(long)]
    pub prompt_file: Option<PathBuf>,

    /// Lê prompt de stdin
    #[arg(long)]
    pub stdin: bool,

    /// Máximo de tokens de resposta
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Output JSON estruturado (com metadados de provider/model/elapsed_ms)
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct ProvidersArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct ChatOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    provider: &'a str,
    model: &'a str,
    elapsed_ms: u128,
    response: String,
}

#[derive(Serialize)]
struct ProvidersOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    providers: Vec<ProviderInfo<'a>>,
}

#[derive(Serialize)]
struct ProviderInfo<'a> {
    name: &'a str,
    default_model: &'a str,
    default_base_url: &'a str,
    auth_env: &'a str,
}

pub fn run(args: Args) -> Result<()> {
    match args.op {
        Op::Chat(a) => run_chat(a),
        Op::Providers(a) => run_providers(a),
    }
}

fn run_chat(args: ChatArgs) -> Result<()> {
    let kind = args
        .provider
        .as_deref()
        .map(ProviderKind::from_str)
        .transpose()
        .map_err(|e| anyhow!("provider inválido: {}", e))?;

    let prompt = resolve_prompt(&args)?;
    if prompt.trim().is_empty() {
        return Err(anyhow!(
            "prompt vazio - use --prompt, --prompt-file ou --stdin"
        ));
    }

    let provider = llm::build(kind, args.model.clone(), args.base_url.clone())
        .map_err(|e| anyhow!("build provider: {}", e))?;

    let mut messages = Vec::new();
    if let Some(sys) = args.system.as_deref() {
        messages.push(ChatMessage::system(sys));
    }
    messages.push(ChatMessage::user(prompt));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let start = std::time::Instant::now();
    let response = rt
        .block_on(provider.chat(&messages, args.max_tokens))
        .map_err(|e| anyhow!("chat: {}", e))?;
    let elapsed_ms = start.elapsed().as_millis();

    if args.json {
        let out = ChatOutput {
            schema: "first-plan-llm-chat-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            provider: provider.name(),
            model: provider.model(),
            elapsed_ms,
            response,
        };
        serde_json::to_writer_pretty(io::stdout().lock(), &out)?;
        writeln!(io::stdout().lock())?;
    } else {
        println!("{}", response);
    }
    Ok(())
}

fn resolve_prompt(args: &ChatArgs) -> Result<String> {
    let choices = [
        args.prompt.is_some(),
        args.prompt_file.is_some(),
        args.stdin,
    ];
    if choices.iter().filter(|c| **c).count() > 1 {
        return Err(anyhow!(
            "escolha apenas uma entre --prompt / --prompt-file / --stdin"
        ));
    }
    if let Some(p) = &args.prompt {
        return Ok(p.clone());
    }
    if let Some(path) = &args.prompt_file {
        return Ok(std::fs::read_to_string(path)?);
    }
    if args.stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    Err(anyhow!("nenhuma fonte de prompt informada"))
}

fn run_providers(args: ProvidersArgs) -> Result<()> {
    let providers = vec![
        ProviderInfo {
            name: "openai",
            default_model: "gpt-4o-mini",
            default_base_url: "https://api.openai.com/v1",
            auth_env: "OPENAI_API_KEY",
        },
        ProviderInfo {
            name: "anthropic",
            default_model: "claude-sonnet-5",
            default_base_url: "https://api.anthropic.com/v1",
            auth_env: "ANTHROPIC_API_KEY",
        },
        ProviderInfo {
            name: "ollama",
            default_model: "qwen2.5-coder:latest",
            default_base_url: "http://localhost:11434/v1",
            auth_env: "(none)",
        },
    ];

    if args.json {
        let out = ProvidersOutput {
            schema: "first-plan-llm-providers-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            providers,
        };
        serde_json::to_writer_pretty(io::stdout().lock(), &out)?;
        writeln!(io::stdout().lock())?;
    } else {
        println!("Providers suportados:");
        for p in &providers {
            println!(
                "  {:<10} model={}  base={}  auth={}",
                p.name, p.default_model, p.default_base_url, p.auth_env
            );
        }
        println!();
        println!("Config via env:");
        println!("  FIRST_PLAN_LLM_PROVIDER=<name>");
        println!("  FIRST_PLAN_LLM_MODEL=<id>");
        println!("  FIRST_PLAN_LLM_BASE_URL=<url>");
    }
    Ok(())
}
