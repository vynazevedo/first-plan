use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_warning,
    OutputMode,
};
use anyhow::{anyhow, Result};
use clap::{Args as ClapArgs, Subcommand};
use crossterm::style::{Color, Stylize};
use first_plan_core::lsp::{
    client::LspClient, daemon, detect_all, detect_server, fallback, ops, registry, server_for_path,
    servers_for_project, ServerId, ServerStatus,
};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub op: Op,
}

#[derive(Subcommand)]
pub enum Op {
    /// Find references for the symbol at file:line:column.
    Refs(PositionArgs),
    /// Resolve definition for the symbol at file:line:column.
    Def(PositionArgs),
    /// List document symbols for a file.
    Symbols(FileArgs),
    /// Show hover info (type + docs) at file:line:column.
    Hover(PositionArgs),
    /// Search workspace symbols by query.
    Wsymbols(QueryArgs),
    /// Show LSP server detection status (which are installed + install commands).
    Status(StatusArgs),
    /// Manage warm-server daemon.
    Daemon(DaemonArgs),
}

#[derive(ClapArgs)]
pub struct PositionArgs {
    /// Path to the source file.
    #[arg(long)]
    pub file: PathBuf,

    /// Zero-indexed line.
    #[arg(long)]
    pub line: u32,

    /// Zero-indexed column.
    #[arg(long)]
    pub col: u32,

    /// Project root (used by LSP initialize).
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Skip LSP and use fallback (grep) directly.
    #[arg(long)]
    pub no_lsp: bool,

    /// Force JSON output even when stdout is a TTY.
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct FileArgs {
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub no_lsp: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct QueryArgs {
    /// Symbol query string.
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// Server to query (default: pick best for project).
    #[arg(long)]
    pub server: Option<String>,
    #[arg(long)]
    pub no_lsp: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct StatusArgs {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(ClapArgs)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Show daemon status (running/stopped, socket path).
    Status,
    /// Stop the daemon if running.
    Stop,
}

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    op: String,
    server: Option<String>,
    used_fallback: bool,
    elapsed_ms: u64,
    data: T,
}

pub fn run(args: Args) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(run_async(args))
}

async fn run_async(args: Args) -> Result<()> {
    match args.op {
        Op::Refs(a) => run_refs(a).await,
        Op::Def(a) => run_def(a).await,
        Op::Symbols(a) => run_symbols(a).await,
        Op::Hover(a) => run_hover(a).await,
        Op::Wsymbols(a) => run_wsymbols(a).await,
        Op::Status(a) => run_status(a),
        Op::Daemon(a) => run_daemon(a),
    }
}

async fn run_refs(a: PositionArgs) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(a.json);
    let server = server_for_path(&a.file);

    let (refs, used_fallback) = match pick_server(server, a.no_lsp) {
        None => {
            let identifier = identifier_at(&a.file, a.line, a.col)?;
            (fallback::references_via_grep(&a.root, &identifier)?, true)
        }
        Some(sid) => match try_refs_with_lsp(sid, &a).await {
            Ok(refs) => (refs, false),
            Err(_) => {
                let identifier = identifier_at(&a.file, a.line, a.col)?;
                (fallback::references_via_grep(&a.root, &identifier)?, true)
            }
        },
    };

    let elapsed = start.elapsed().as_millis() as u64;
    emit(
        mode,
        Envelope {
            op: "references".into(),
            server: server.map(|s| registry::spec(s).name.to_string()),
            used_fallback,
            elapsed_ms: elapsed,
            data: &refs,
        },
        |refs| {
            print_header("LSP References");
            if used_fallback {
                print_warning("LSP indisponivel, usando fallback grep");
            }
            print_kv_bold("Matches", &refs.len().to_string(), Color::Green);
            print_section("Results");
            for r in refs.iter().take(50) {
                println!(
                    "  {}:{}:{}",
                    r.location.file.as_str().with(Color::Cyan),
                    (r.location.line + 1).to_string().with(Color::Yellow),
                    r.location.column + 1
                );
                if let Some(s) = &r.snippet {
                    println!("      {}", s.trim().dim());
                }
            }
            if refs.len() > 50 {
                println!("  {}", format!("... and {} more", refs.len() - 50).dim());
            }
            print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
            flush();
        },
        &refs,
    )
}

async fn try_refs_with_lsp(sid: ServerId, a: &PositionArgs) -> Result<Vec<ops::Reference>> {
    let client = LspClient::spawn(sid, &a.root).await?;
    let result = ops::references(&client, &a.file, a.line, a.col, true).await;
    let _ = client.shutdown().await;
    result
}

async fn run_def(a: PositionArgs) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(a.json);
    let server = server_for_path(&a.file);

    let (defs, used_fallback) = match pick_server(server, a.no_lsp) {
        None => {
            let identifier = identifier_at(&a.file, a.line, a.col)?;
            let refs = fallback::references_via_grep(&a.root, &identifier)?;
            let defs: Vec<_> = refs
                .into_iter()
                .filter(|r| {
                    r.snippet
                        .as_ref()
                        .map(|s| {
                            s.contains("fn ")
                                || s.contains("func ")
                                || s.contains("def ")
                                || s.contains("class ")
                                || s.contains("struct ")
                                || s.contains("type ")
                                || s.contains("const ")
                                || s.contains("let ")
                                || s.contains("var ")
                        })
                        .unwrap_or(false)
                })
                .map(|r| r.location)
                .collect();
            (defs, true)
        }
        Some(sid) => {
            let client = LspClient::spawn(sid, &a.root).await?;
            let r = ops::definition(&client, &a.file, a.line, a.col).await?;
            let _ = client.shutdown().await;
            (r, false)
        }
    };

    let elapsed = start.elapsed().as_millis() as u64;
    emit(
        mode,
        Envelope {
            op: "definition".into(),
            server: server.map(|s| registry::spec(s).name.to_string()),
            used_fallback,
            elapsed_ms: elapsed,
            data: &defs,
        },
        |defs| {
            print_header("LSP Definition");
            if used_fallback {
                print_warning("LSP indisponivel, usando fallback heuristico");
            }
            if defs.is_empty() {
                print_warning("Nenhuma definicao encontrada");
            } else {
                for d in defs {
                    println!(
                        "  {}:{}:{}",
                        d.file.as_str().with(Color::Cyan),
                        (d.line + 1).to_string().with(Color::Yellow),
                        d.column + 1
                    );
                }
            }
            print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
            flush();
        },
        &defs,
    )
}

async fn run_symbols(a: FileArgs) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(a.json);
    let server = server_for_path(&a.file);

    let (syms, used_fallback) = match pick_server(server, a.no_lsp) {
        None => {
            let parent = a.file.parent().unwrap_or(&a.root);
            let syms = fallback::workspace_symbols_via_grep(parent, "")?
                .into_iter()
                .filter(|s| s.location.file == a.file.to_string_lossy())
                .collect();
            (syms, true)
        }
        Some(sid) => {
            let client = LspClient::spawn(sid, &a.root).await?;
            let r = ops::document_symbol(&client, &a.file).await?;
            let _ = client.shutdown().await;
            (r, false)
        }
    };

    let elapsed = start.elapsed().as_millis() as u64;
    emit(
        mode,
        Envelope {
            op: "documentSymbol".into(),
            server: server.map(|s| registry::spec(s).name.to_string()),
            used_fallback,
            elapsed_ms: elapsed,
            data: &syms,
        },
        |syms| {
            print_header(&format!("Symbols in {}", a.file.display()));
            if used_fallback {
                print_warning("LSP indisponivel, usando fallback");
            }
            print_kv_bold("Total", &syms.len().to_string(), Color::Green);
            print_section("Symbols");
            for s in syms.iter().take(100) {
                let kind = format!("{:?}", s.kind).to_lowercase();
                println!(
                    "  {} {} {}",
                    format!("L{}", s.location.line + 1).with(Color::Yellow),
                    s.name.as_str().bold().with(Color::Cyan),
                    format!("[{}]", kind).dim()
                );
            }
            if syms.len() > 100 {
                println!("  {}", format!("... and {} more", syms.len() - 100).dim());
            }
            print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
            flush();
        },
        &syms,
    )
}

async fn run_hover(a: PositionArgs) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(a.json);
    let server = server_for_path(&a.file);

    let (content, used_fallback) = match pick_server(server, a.no_lsp) {
        None => {
            let snippet = fallback::hover_via_source(&a.file, a.line)?;
            (snippet.unwrap_or_default(), true)
        }
        Some(sid) => {
            let client = LspClient::spawn(sid, &a.root).await?;
            let r = ops::hover(&client, &a.file, a.line, a.col).await?;
            let _ = client.shutdown().await;
            (r.map(|h| h.content).unwrap_or_default(), false)
        }
    };

    let elapsed = start.elapsed().as_millis() as u64;
    emit(
        mode,
        Envelope {
            op: "hover".into(),
            server: server.map(|s| registry::spec(s).name.to_string()),
            used_fallback,
            elapsed_ms: elapsed,
            data: &content,
        },
        |content| {
            print_header("LSP Hover");
            if used_fallback {
                print_warning("LSP indisponivel, usando snippet do source");
            }
            if content.is_empty() {
                print_warning("Sem informacao");
            } else {
                println!("{}", content);
            }
            print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
            flush();
        },
        &content,
    )
}

async fn run_wsymbols(a: QueryArgs) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(a.json);

    let server_id = if let Some(name) = &a.server {
        ServerId::all()
            .iter()
            .copied()
            .find(|id| registry::spec(*id).name == name.as_str())
    } else {
        servers_for_project(&a.root).into_iter().next()
    };

    let (syms, used_fallback) = match pick_server(server_id, a.no_lsp) {
        None => (
            fallback::workspace_symbols_via_grep(&a.root, &a.query)?,
            true,
        ),
        Some(sid) => {
            let client = LspClient::spawn(sid, &a.root).await?;
            let r = ops::workspace_symbol(&client, &a.query).await?;
            let _ = client.shutdown().await;
            (r, false)
        }
    };

    let elapsed = start.elapsed().as_millis() as u64;
    emit(
        mode,
        Envelope {
            op: "workspaceSymbol".into(),
            server: server_id.map(|s| registry::spec(s).name.to_string()),
            used_fallback,
            elapsed_ms: elapsed,
            data: &syms,
        },
        |syms| {
            print_header(&format!("Workspace symbols: {}", a.query));
            if used_fallback {
                print_warning("LSP indisponivel, usando fallback grep");
            }
            print_kv_bold("Matches", &syms.len().to_string(), Color::Green);
            print_section("Results");
            for s in syms.iter().take(50) {
                let kind = format!("{:?}", s.kind).to_lowercase();
                println!(
                    "  {} {} {} {}",
                    s.name.as_str().bold().with(Color::Cyan),
                    format!("[{}]", kind).dim(),
                    "in".dim(),
                    s.location.file.as_str().with(Color::White)
                );
            }
            if syms.len() > 50 {
                println!("  {}", format!("... and {} more", syms.len() - 50).dim());
            }
            print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
            flush();
        },
        &syms,
    )
}

fn run_status(a: StatusArgs) -> Result<()> {
    let mode = output_mode(a.json);
    let all = detect_all();
    let project_servers = servers_for_project(&a.root);

    if mode == OutputMode::Json {
        let payload = serde_json::json!({
            "engine_version": env!("CARGO_PKG_VERSION"),
            "project_root": a.root.to_string_lossy(),
            "project_needs": project_servers
                .iter()
                .map(|s| registry::spec(*s).name)
                .collect::<Vec<_>>(),
            "servers": all,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    print_header("LSP servers");
    print_section("Status");

    for s in &all {
        let mark = if s.installed { "ok" } else { "--" };
        let mark_styled = if s.installed {
            mark.with(Color::Green)
        } else {
            mark.with(Color::DarkGrey)
        };
        let needed_here = project_servers.contains(&s.id);
        let need_mark = if needed_here {
            "(needed)".with(Color::Yellow)
        } else {
            "".with(Color::DarkGrey)
        };
        println!(
            "  {} {:<28} {:<22} {}",
            mark_styled,
            s.name.as_str().bold(),
            s.language.as_str().dim(),
            need_mark
        );
        if s.installed {
            if let Some(p) = &s.path {
                println!("      {} {}", "path:".dim(), p.as_str().dim());
            }
            if let Some(v) = &s.version {
                println!("      {} {}", "version:".dim(), v.as_str().dim());
            }
        } else {
            println!(
                "      {} {}",
                "install:".dim(),
                s.install_cmd.as_str().dim()
            );
        }
    }

    print_section("Project");
    if project_servers.is_empty() {
        println!("  nenhuma stack detectada via manifests");
    } else {
        for sid in &project_servers {
            let st = detect_server(*sid);
            let mark = if st.installed { "ok" } else { "instalar" };
            println!(
                "  {} {} ({})",
                mark.with(if st.installed {
                    Color::Green
                } else {
                    Color::Yellow
                }),
                st.name.as_str().bold(),
                st.language.as_str().dim()
            );
        }
    }
    flush();
    Ok(())
}

fn run_daemon(a: DaemonArgs) -> Result<()> {
    match a.action {
        DaemonAction::Status => {
            let s = daemon::status();
            print_header("Daemon status");
            print_kv_bold(
                "Running",
                if s.running { "yes" } else { "no" },
                if s.running {
                    Color::Green
                } else {
                    Color::DarkGrey
                },
            );
            print_kv("Socket", &s.socket_path, Color::DarkGrey);
            print_kv("PID file", &s.pid_file, Color::DarkGrey);
            flush();
            Ok(())
        }
        DaemonAction::Stop => {
            daemon::stop()?;
            println!("Daemon stopped (if running)");
            Ok(())
        }
    }
}

fn server_installed(id: ServerId) -> bool {
    detect_server(id).installed
}

fn pick_server(server: Option<ServerId>, no_lsp: bool) -> Option<ServerId> {
    if no_lsp {
        return None;
    }
    let id = server?;
    if !server_installed(id) {
        return None;
    }
    Some(id)
}

fn identifier_at(file: &std::path::Path, line: u32, col: u32) -> Result<String> {
    let content = std::fs::read_to_string(file)?;
    let line_str = content
        .lines()
        .nth(line as usize)
        .ok_or_else(|| anyhow!("line {} out of range", line))?;
    let bytes = line_str.as_bytes();
    let c = col as usize;
    if c >= bytes.len() {
        return Err(anyhow!("column {} out of range on line {}", col, line));
    }
    let is_id_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut start = c;
    while start > 0 && is_id_char(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = c;
    while end < bytes.len() && is_id_char(bytes[end]) {
        end += 1;
    }
    if start == end {
        return Err(anyhow!("no identifier at {}:{}", line, col));
    }
    Ok(line_str[start..end].to_string())
}

fn emit<T: Serialize, F: FnOnce(&T)>(
    mode: OutputMode,
    env: Envelope<&T>,
    pretty_fn: F,
    data: &T,
) -> Result<()> {
    match mode {
        OutputMode::Json => {
            println!("{}", serde_json::to_string_pretty(&env)?);
            Ok(())
        }
        OutputMode::Pretty => {
            pretty_fn(data);
            Ok(())
        }
    }
}

#[allow(dead_code)]
fn _suppress_unused(_a: &ServerStatus) {}
