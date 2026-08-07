use anyhow::{anyhow, Result};
use clap::{Args as ClapArgs, Subcommand};
use first_plan_core::multirepo::{self, MultiRepoConfig, RepoEntry};
use serde::Serialize;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub op: Op,
}

#[derive(Subcommand)]
pub enum Op {
    /// Registra um sibling repo em .first-plan/multi.yaml.
    Register(RegisterArgs),
    /// Lista repos registrados com status (existe? tem IR?).
    List(ListArgs),
    /// Escaneia diretório-pai para autodetectar sibling repos.
    Scan(ScanArgs),
    /// Agrega IR de todos os repos registrados em .first-plan/multi/OVERVIEW.md.
    Aggregate(AggregateArgs),
    /// Remove um repo do registro por nome.
    Remove(RemoveArgs),
    /// Roda contracts diff em cada repo registrado contra seu snapshot baseline.
    ContractsCheck(ContractsCheckArgs),
}

#[derive(ClapArgs)]
pub struct RegisterArgs {
    /// Nome canônico do repo (usado como chave no registro)
    #[arg(long)]
    pub name: String,
    /// Path do repo (relativo ao --root ou absoluto)
    #[arg(long)]
    pub path: PathBuf,
    /// Tags opcionais (repetível)
    #[arg(long)]
    pub tag: Vec<String>,
    /// Notas em texto livre
    #[arg(long)]
    pub notes: Option<String>,
    /// Project root
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct ListArgs {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct ScanArgs {
    /// Diretório-pai onde procurar por sibling repos
    #[arg(long)]
    pub parent: PathBuf,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// Se presente, registra todos os detectados automaticamente
    #[arg(long)]
    pub register_all: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct AggregateArgs {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct RemoveArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct ContractsCheckArgs {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// Path relativo ao repo onde o snapshot baseline é procurado
    #[arg(long, default_value = ".first-plan/12-contracts/snapshot.json")]
    pub baseline: PathBuf,
    /// Falha com exit code 1 se qualquer repo tiver breaking changes
    #[arg(long)]
    pub fail_on_breaking: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct RegisterOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    action: &'a str,
    name: &'a str,
    config_path: PathBuf,
    total_repos: usize,
}

#[derive(Serialize)]
struct ListOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    config_path: PathBuf,
    total_repos: usize,
    repos: Vec<ListEntry>,
}

#[derive(Serialize)]
struct ListEntry {
    name: String,
    path: PathBuf,
    resolved_path: PathBuf,
    tags: Vec<String>,
    exists: bool,
    has_first_plan: bool,
}

#[derive(Serialize)]
struct ScanOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    parent: PathBuf,
    detected: Vec<ScanEntry>,
    registered: Vec<String>,
}

#[derive(Serialize)]
struct ScanEntry {
    name: String,
    path: PathBuf,
    has_first_plan: bool,
    detected_stacks: Vec<String>,
}

pub fn run(args: Args) -> Result<()> {
    match args.op {
        Op::Register(a) => run_register(a),
        Op::List(a) => run_list(a),
        Op::Scan(a) => run_scan(a),
        Op::Aggregate(a) => run_aggregate(a),
        Op::Remove(a) => run_remove(a),
        Op::ContractsCheck(a) => run_contracts_check(a),
    }
}

fn run_register(args: RegisterArgs) -> Result<()> {
    let mut cfg = multirepo::load(&args.root)?;
    if cfg.repos.iter().any(|r| r.name == args.name) {
        return Err(anyhow!(
            "repo '{}' já registrado (use `multi remove --name {}` primeiro)",
            args.name,
            args.name
        ));
    }
    cfg.repos.push(RepoEntry {
        name: args.name.clone(),
        path: args.path.clone(),
        tags: args.tag.clone(),
        notes: args.notes.clone(),
    });
    let path = multirepo::save(&args.root, &cfg)?;

    if args.json {
        let out = RegisterOutput {
            schema: "first-plan-multi-register-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            action: "registered",
            name: &args.name,
            config_path: path,
            total_repos: cfg.repos.len(),
        };
        serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
        println!();
    } else {
        println!(
            "Registrado '{}' em {}. Total: {}.",
            args.name,
            path.display(),
            cfg.repos.len()
        );
    }
    Ok(())
}

fn run_list(args: ListArgs) -> Result<()> {
    let cfg = multirepo::load(&args.root)?;
    let cfg_path = multirepo::config_path(&args.root);

    let entries: Vec<ListEntry> = cfg
        .repos
        .iter()
        .map(|r| {
            let resolved = multirepo::resolved_path(&args.root, r);
            let exists = resolved.exists();
            let has_first_plan = resolved.join(".first-plan").is_dir();
            ListEntry {
                name: r.name.clone(),
                path: r.path.clone(),
                resolved_path: resolved,
                tags: r.tags.clone(),
                exists,
                has_first_plan,
            }
        })
        .collect();

    if args.json {
        let out = ListOutput {
            schema: "first-plan-multi-list-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            config_path: cfg_path,
            total_repos: entries.len(),
            repos: entries,
        };
        serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
        println!();
    } else {
        println!("Config: {}", cfg_path.display());
        if entries.is_empty() {
            println!("Nenhum repo registrado.");
            return Ok(());
        }
        println!("{} repo(s) registrado(s):", entries.len());
        for e in &entries {
            println!(
                "  {:<20} path={:<40} exists={} ir={} tags={}",
                e.name,
                e.resolved_path.display().to_string(),
                if e.exists { "sim" } else { "não" },
                if e.has_first_plan { "sim" } else { "não" },
                if e.tags.is_empty() {
                    "-".to_string()
                } else {
                    e.tags.join(",")
                }
            );
        }
    }
    Ok(())
}

fn run_scan(args: ScanArgs) -> Result<()> {
    let detected = multirepo::scan(&args.parent, &args.root)?;

    let mut registered_names = Vec::new();
    if args.register_all && !detected.is_empty() {
        let mut cfg = multirepo::load(&args.root)?;
        for d in &detected {
            if cfg.repos.iter().any(|r| r.name == d.name) {
                continue;
            }
            cfg.repos.push(RepoEntry {
                name: d.name.clone(),
                path: d.path.clone(),
                tags: d.detected_stacks.clone(),
                notes: Some(format!(
                    "Auto-registrado via `multi scan` em {}",
                    args.parent.display()
                )),
            });
            registered_names.push(d.name.clone());
        }
        multirepo::save(&args.root, &cfg)?;
    }

    if args.json {
        let out = ScanOutput {
            schema: "first-plan-multi-scan-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            parent: args.parent.clone(),
            detected: detected
                .iter()
                .map(|d| ScanEntry {
                    name: d.name.clone(),
                    path: d.path.clone(),
                    has_first_plan: d.has_first_plan,
                    detected_stacks: d.detected_stacks.clone(),
                })
                .collect(),
            registered: registered_names,
        };
        serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
        println!();
    } else {
        println!("Scan em {}:", args.parent.display());
        println!("Detectados: {}", detected.len());
        for d in &detected {
            println!(
                "  {:<20} path={:<40} ir={} stacks=[{}]",
                d.name,
                d.path.display().to_string(),
                if d.has_first_plan { "sim" } else { "não" },
                d.detected_stacks.join(",")
            );
        }
        if !registered_names.is_empty() {
            println!("Auto-registrados: {}", registered_names.join(", "));
        } else if args.register_all {
            println!("Nada novo para registrar.");
        }
    }
    Ok(())
}

fn run_aggregate(args: AggregateArgs) -> Result<()> {
    let cfg = multirepo::load(&args.root)?;
    let report = multirepo::aggregate(&args.root, &cfg)?;

    if args.json {
        serde_json::to_writer_pretty(std::io::stdout().lock(), &report)?;
        println!();
    } else {
        println!(
            "Overview cross-repo gerado em {} ({} repos).",
            report.output_path.display(),
            report.repos.len()
        );
        for r in &report.repos {
            println!(
                "  {:<20} exists={} ir={} layers={}",
                r.name,
                if r.exists { "sim" } else { "não" },
                if r.has_first_plan { "sim" } else { "não" },
                r.layers_found.len()
            );
        }
    }
    Ok(())
}

fn run_remove(args: RemoveArgs) -> Result<()> {
    let mut cfg = multirepo::load(&args.root)?;
    let before = cfg.repos.len();
    cfg.repos.retain(|r| r.name != args.name);
    if cfg.repos.len() == before {
        return Err(anyhow!("repo '{}' não encontrado no registro", args.name));
    }
    let path = multirepo::save(&args.root, &cfg)?;

    if args.json {
        let out = RegisterOutput {
            schema: "first-plan-multi-register-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            action: "removed",
            name: &args.name,
            config_path: path,
            total_repos: cfg.repos.len(),
        };
        serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
        println!();
    } else {
        println!(
            "Removido '{}'. Total agora: {} repo(s).",
            args.name,
            cfg.repos.len()
        );
    }
    Ok(())
}

#[derive(Serialize)]
struct ContractsCheckOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'a str,
    engine_version: &'a str,
    total_repos: usize,
    checked: usize,
    skipped: usize,
    total_breaking: usize,
    repos: Vec<RepoCheckResult>,
}

#[derive(Serialize)]
struct RepoCheckResult {
    name: String,
    path: PathBuf,
    status: String,
    baseline_path: Option<PathBuf>,
    total_changes: usize,
    breaking: usize,
    non_breaking: usize,
    reason: Option<String>,
}

fn run_contracts_check(args: ContractsCheckArgs) -> Result<()> {
    use first_plan_core::contracts::{analyze, diff, ContractsReport};

    let cfg = multirepo::load(&args.root)?;
    let mut results = Vec::new();
    let mut checked = 0usize;
    let mut skipped = 0usize;
    let mut total_breaking = 0usize;

    for entry in &cfg.repos {
        let repo_path = multirepo::resolved_path(&args.root, entry);
        let baseline_path = repo_path.join(&args.baseline);

        if !repo_path.exists() {
            skipped += 1;
            results.push(RepoCheckResult {
                name: entry.name.clone(),
                path: repo_path.clone(),
                status: "skipped".to_string(),
                baseline_path: None,
                total_changes: 0,
                breaking: 0,
                non_breaking: 0,
                reason: Some("repo path não existe".to_string()),
            });
            continue;
        }
        if !baseline_path.exists() {
            skipped += 1;
            results.push(RepoCheckResult {
                name: entry.name.clone(),
                path: repo_path.clone(),
                status: "skipped".to_string(),
                baseline_path: Some(baseline_path),
                total_changes: 0,
                breaking: 0,
                non_breaking: 0,
                reason: Some(
                    "baseline não encontrado (rodar `contracts snapshot` no repo primeiro)"
                        .to_string(),
                ),
            });
            continue;
        }

        let baseline_text = std::fs::read_to_string(&baseline_path)?;
        let before: ContractsReport = serde_json::from_str(&baseline_text)?;
        let after = analyze(&repo_path);
        let d = diff::diff(&before, &after);
        total_breaking += d.summary.breaking;
        checked += 1;

        results.push(RepoCheckResult {
            name: entry.name.clone(),
            path: repo_path.clone(),
            status: if d.summary.breaking > 0 {
                "breaking".to_string()
            } else if d.summary.total_changes > 0 {
                "changed".to_string()
            } else {
                "clean".to_string()
            },
            baseline_path: Some(baseline_path),
            total_changes: d.summary.total_changes,
            breaking: d.summary.breaking,
            non_breaking: d.summary.non_breaking,
            reason: None,
        });
    }

    let breaking_repo_count = results.iter().filter(|r| r.breaking > 0).count();

    if args.json {
        let out = ContractsCheckOutput {
            schema: "first-plan-multi-contracts-check-v1",
            engine_version: first_plan_core::ENGINE_VERSION,
            total_repos: cfg.repos.len(),
            checked,
            skipped,
            total_breaking,
            repos: results,
        };
        serde_json::to_writer_pretty(std::io::stdout().lock(), &out)?;
        println!();
    } else {
        println!(
            "Contracts check em {} repo(s) registrado(s): {} checked, {} skipped, {} breaking total.",
            cfg.repos.len(),
            checked,
            skipped,
            total_breaking
        );
        for r in &results {
            let marker = match r.status.as_str() {
                "breaking" => "!",
                "changed" => "~",
                "clean" => ".",
                _ => "?",
            };
            print!("  {} {:<20} status={:<9}", marker, r.name, r.status);
            if r.status == "skipped" {
                if let Some(reason) = &r.reason {
                    println!(" reason={}", reason);
                } else {
                    println!();
                }
            } else {
                println!(
                    " changes={} breaking={} non-breaking={}",
                    r.total_changes, r.breaking, r.non_breaking
                );
            }
        }
    }

    if args.fail_on_breaking && total_breaking > 0 {
        return Err(anyhow!(
            "{} breaking change(s) detectados em {} repo(s) (--fail-on-breaking)",
            total_breaking,
            breaking_repo_count
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn _fmt_cfg(_c: &MultiRepoConfig) {}
