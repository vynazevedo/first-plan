//! Daemon mode - mantem pool de LSP servers warm via Unix socket.
//!
//! Reduz cold start de 3-15s para <100ms apos primeira invocacao por server.
//! Em sessoes longas, primeira chamada paga o custo de spawn + indexing,
//! demais sao instantaneas pois o LspClient ja esta vivo no daemon.
//!
//! Protocolo IPC (line-delimited JSON sobre Unix socket):
//!
//! Request:  {"id": N, "op": "refs"|"def"|"symbols"|"hover"|"wsymbols"|"status"|"shutdown", "args": {...}}
//! Response: {"id": N, "result": <Value>} | {"id": N, "error": "msg"}
//!
//! Daemon detecta server apropriado por arquivo (ServerId baseado em extensao)
//! e mantem um LspClient por ServerId, criado sob demanda (lazy spawn).

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::client::LspClient;
use super::ops;
use super::registry::{server_for_path, ServerId};

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub id: u64,
    pub op: String,
    pub args: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub socket_path: String,
    pub pid_file: String,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub idle_seconds: Option<u64>,
    pub warm_servers: Vec<String>,
}

pub fn socket_path() -> PathBuf {
    let dir = dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(std::env::temp_dir);
    dir.join("first-plan-engine.sock")
}

pub fn pid_path() -> PathBuf {
    let dir = dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(std::env::temp_dir);
    dir.join("first-plan-engine.pid")
}

pub fn read_pid() -> Option<u32> {
    let content = std::fs::read_to_string(pid_path()).ok()?;
    content.trim().parse().ok()
}

pub fn is_running() -> bool {
    let Some(pid) = read_pid() else {
        return false;
    };
    process_alive(pid)
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    if Path::new(&format!("/proc/{}", pid)).exists() {
        return true;
    }
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn process_alive(_pid: u32) -> bool {
    false
}

pub async fn status() -> DaemonStatus {
    let pid = read_pid();
    let running = pid.map(process_alive).unwrap_or(false);

    let (uptime, idle, warm) = if running {
        match query_status().await {
            Ok(s) => (s.uptime_seconds, s.idle_seconds, s.warm_servers),
            Err(_) => (None, None, Vec::new()),
        }
    } else {
        (None, None, Vec::new())
    };

    DaemonStatus {
        running,
        socket_path: socket_path().to_string_lossy().into_owned(),
        pid_file: pid_path().to_string_lossy().into_owned(),
        pid,
        uptime_seconds: uptime,
        idle_seconds: idle,
        warm_servers: warm,
    }
}

async fn query_status() -> Result<DaemonStatus> {
    let raw = send_request(&DaemonRequest {
        id: 1,
        op: "status".into(),
        args: Value::Null,
    })
    .await?;
    let st: DaemonStatus = serde_json::from_value(raw)?;
    Ok(st)
}

pub async fn stop() -> Result<()> {
    if is_running() {
        let _ = send_request(&DaemonRequest {
            id: 1,
            op: "shutdown".into(),
            args: Value::Null,
        })
        .await;
        for _ in 0..20 {
            if !is_running() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        if is_running() {
            if let Some(pid) = read_pid() {
                #[cfg(unix)]
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
        }
    }
    let _ = std::fs::remove_file(pid_path());
    let _ = std::fs::remove_file(socket_path());
    Ok(())
}

pub async fn send_request(req: &DaemonRequest) -> Result<Value> {
    let sock = socket_path();
    let stream = timeout(Duration::from_secs(2), UnixStream::connect(&sock))
        .await
        .map_err(|_| anyhow!("daemon socket connect timeout"))?
        .with_context(|| format!("failed to connect to daemon socket {}", sock.display()))?;

    let (read_half, mut write_half) = stream.into_split();
    let body = serde_json::to_string(req)?;
    write_half.write_all(body.as_bytes()).await?;
    write_half.write_all(b"\n").await?;
    write_half.flush().await?;

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    timeout(Duration::from_secs(60), reader.read_line(&mut line))
        .await
        .map_err(|_| anyhow!("daemon response timeout"))??;

    let resp: DaemonResponse = serde_json::from_str(line.trim())
        .with_context(|| format!("daemon returned invalid JSON: {}", line.trim()))?;

    if let Some(err) = resp.error {
        return Err(anyhow!("daemon: {}", err));
    }
    resp.result
        .ok_or_else(|| anyhow!("daemon response missing both result and error"))
}

pub struct Daemon {
    root: PathBuf,
    idle_timeout: Duration,
    clients: Arc<Mutex<HashMap<ServerId, Arc<LspClient>>>>,
    started_at: Instant,
    last_request: Arc<Mutex<Instant>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Daemon {
    pub fn new(root: PathBuf, idle_timeout: Duration) -> Self {
        let now = Instant::now();
        Self {
            root,
            idle_timeout,
            clients: Arc::new(Mutex::new(HashMap::new())),
            started_at: now,
            last_request: Arc::new(Mutex::new(now)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn run(self) -> Result<()> {
        let sock = socket_path();
        let pid = pid_path();

        if is_running() {
            return Err(anyhow!("daemon ja esta rodando (pid em {})", pid.display()));
        }

        if sock.exists() {
            let _ = std::fs::remove_file(&sock);
        }

        std::fs::write(&pid, std::process::id().to_string())?;
        let listener = UnixListener::bind(&sock)
            .with_context(|| format!("failed to bind unix socket at {}", sock.display()))?;

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        let daemon = Arc::new(self);
        let daemon_idle = daemon.clone();

        let idle_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let last = *daemon_idle.last_request.lock().await;
                if last.elapsed() >= daemon_idle.idle_timeout {
                    eprintln!("daemon idle for {:?}, shutting down", last.elapsed());
                    if let Some(tx) = daemon_idle.shutdown_tx.lock().await.take() {
                        let _ = tx.send(());
                    }
                    break;
                }
            }
        });

        let daemon_accept = daemon.clone();
        let accept_task = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let d = daemon_accept.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(d, stream).await {
                                eprintln!("daemon connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("daemon accept error: {}", e);
                        break;
                    }
                }
            }
        });

        let _ = shutdown_rx.await;
        idle_task.abort();
        accept_task.abort();

        let mut clients = daemon.clients.lock().await;
        let pairs: Vec<(ServerId, Arc<LspClient>)> = clients.drain().collect();
        drop(clients);
        for (_, c) in pairs {
            if let Ok(c) = Arc::try_unwrap(c) {
                let _ = c.shutdown().await;
            }
        }

        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_file(&pid);
        Ok(())
    }

    async fn touch(&self) {
        *self.last_request.lock().await = Instant::now();
    }

    async fn get_or_spawn_client(&self, server_id: ServerId) -> Result<Arc<LspClient>> {
        {
            let map = self.clients.lock().await;
            if let Some(c) = map.get(&server_id) {
                return Ok(c.clone());
            }
        }
        let client = LspClient::spawn(server_id, &self.root).await?;
        let arc = Arc::new(client);
        let mut map = self.clients.lock().await;
        let entry = map.entry(server_id).or_insert_with(|| arc.clone());
        Ok(entry.clone())
    }
}

async fn handle_connection(daemon: Arc<Daemon>, stream: UnixStream) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let req: DaemonRequest = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(e) => {
            let resp = DaemonResponse {
                id: 0,
                result: None,
                error: Some(format!("invalid request: {}", e)),
            };
            let body = serde_json::to_string(&resp)?;
            write_half.write_all(body.as_bytes()).await?;
            write_half.write_all(b"\n").await?;
            return Ok(());
        }
    };

    daemon.touch().await;

    let response = process_request(daemon.clone(), &req).await;
    let resp = match response {
        Ok(v) => DaemonResponse {
            id: req.id,
            result: Some(v),
            error: None,
        },
        Err(e) => DaemonResponse {
            id: req.id,
            result: None,
            error: Some(e.to_string()),
        },
    };

    let body = serde_json::to_string(&resp)?;
    write_half.write_all(body.as_bytes()).await?;
    write_half.write_all(b"\n").await?;
    write_half.flush().await?;
    Ok(())
}

async fn process_request(daemon: Arc<Daemon>, req: &DaemonRequest) -> Result<Value> {
    match req.op.as_str() {
        "status" => {
            let warm = daemon
                .clients
                .lock()
                .await
                .keys()
                .map(|id| super::registry::spec(*id).name.to_string())
                .collect::<Vec<_>>();
            let last = *daemon.last_request.lock().await;
            Ok(json!({
                "running": true,
                "socket_path": socket_path().to_string_lossy(),
                "pid_file": pid_path().to_string_lossy(),
                "pid": std::process::id(),
                "uptime_seconds": daemon.started_at.elapsed().as_secs(),
                "idle_seconds": last.elapsed().as_secs(),
                "warm_servers": warm,
            }))
        }
        "shutdown" => {
            if let Some(tx) = daemon.shutdown_tx.lock().await.take() {
                let _ = tx.send(());
            }
            Ok(json!({ "ok": true }))
        }
        "refs" => {
            let file: PathBuf = parse_arg(&req.args, "file")?;
            let line = parse_u32(&req.args, "line")?;
            let col = parse_u32(&req.args, "col")?;
            let include = req
                .args
                .get("include_declaration")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let sid = server_for_path(&file)
                .ok_or_else(|| anyhow!("no server registered for file extension"))?;
            let client = daemon.get_or_spawn_client(sid).await?;
            let r = ops::references(&client, &file, line, col, include).await?;
            Ok(serde_json::to_value(r)?)
        }
        "def" => {
            let file: PathBuf = parse_arg(&req.args, "file")?;
            let line = parse_u32(&req.args, "line")?;
            let col = parse_u32(&req.args, "col")?;
            let sid = server_for_path(&file)
                .ok_or_else(|| anyhow!("no server registered for file extension"))?;
            let client = daemon.get_or_spawn_client(sid).await?;
            let r = ops::definition(&client, &file, line, col).await?;
            Ok(serde_json::to_value(r)?)
        }
        "symbols" => {
            let file: PathBuf = parse_arg(&req.args, "file")?;
            let sid = server_for_path(&file)
                .ok_or_else(|| anyhow!("no server registered for file extension"))?;
            let client = daemon.get_or_spawn_client(sid).await?;
            let r = ops::document_symbol(&client, &file).await?;
            Ok(serde_json::to_value(r)?)
        }
        "hover" => {
            let file: PathBuf = parse_arg(&req.args, "file")?;
            let line = parse_u32(&req.args, "line")?;
            let col = parse_u32(&req.args, "col")?;
            let sid = server_for_path(&file)
                .ok_or_else(|| anyhow!("no server registered for file extension"))?;
            let client = daemon.get_or_spawn_client(sid).await?;
            let r = ops::hover(&client, &file, line, col).await?;
            Ok(serde_json::to_value(r)?)
        }
        "wsymbols" => {
            let query = req
                .args
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let server_name = req.args.get("server").and_then(|v| v.as_str());
            let sid = match server_name {
                Some(name) => ServerId::all()
                    .iter()
                    .copied()
                    .find(|id| super::registry::spec(*id).name == name)
                    .ok_or_else(|| anyhow!("unknown server name: {}", name))?,
                None => super::registry::servers_for_project(&daemon.root)
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("no stack detected at daemon root"))?,
            };
            let client = daemon.get_or_spawn_client(sid).await?;
            let r = ops::workspace_symbol(&client, &query).await?;
            Ok(serde_json::to_value(r)?)
        }
        op => Err(anyhow!("unknown daemon op: {}", op)),
    }
}

fn parse_arg<T: for<'de> Deserialize<'de>>(args: &Value, key: &str) -> Result<T> {
    let v = args
        .get(key)
        .ok_or_else(|| anyhow!("missing arg: {}", key))?;
    Ok(serde_json::from_value(v.clone())?)
}

fn parse_u32(args: &Value, key: &str) -> Result<u32> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        .ok_or_else(|| anyhow!("missing or invalid u32 arg: {}", key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_response_roundtrip() {
        let req = DaemonRequest {
            id: 42,
            op: "refs".into(),
            args: json!({"file": "/tmp/x.rs", "line": 0, "col": 5}),
        };
        let json_str = serde_json::to_string(&req).unwrap();
        let parsed: DaemonRequest = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.id, 42);
        assert_eq!(parsed.op, "refs");
        assert_eq!(parsed.args.get("line").and_then(|v| v.as_u64()), Some(0));
    }

    #[test]
    fn response_with_error_serializes() {
        let resp = DaemonResponse {
            id: 1,
            result: None,
            error: Some("test error".into()),
        };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("\"error\""));
        assert!(!s.contains("\"result\""));
    }

    #[test]
    fn response_with_result_serializes() {
        let resp = DaemonResponse {
            id: 1,
            result: Some(json!([1, 2, 3])),
            error: None,
        };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("\"result\""));
        assert!(!s.contains("\"error\""));
    }

    #[test]
    fn parse_u32_handles_valid() {
        let v = json!({"line": 42});
        assert_eq!(parse_u32(&v, "line").unwrap(), 42);
    }

    #[test]
    fn parse_u32_errors_on_missing() {
        let v = json!({});
        assert!(parse_u32(&v, "line").is_err());
    }
}
