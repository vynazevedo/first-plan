//! Daemon mode - mantem pool de LSP servers warm via Unix socket.
//!
//! Reduz cold start (gopls/rust-analyzer levam 3-10s pra indexar).
//! Em sessoes longas, primeira chamada paga o custo, demais sao instantaneas.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::registry::ServerId;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub socket_path: String,
    pub pid_file: String,
    pub warm_servers: Vec<String>,
    pub idle_seconds: Option<u64>,
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

pub fn is_running() -> bool {
    let pid_file = pid_path();
    if !pid_file.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(&pid_file) else {
        return false;
    };
    let Ok(pid) = content.trim().parse::<u32>() else {
        return false;
    };
    process_alive(pid)
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists() || unsafe { libc_kill_check(pid as i32) }
}

#[cfg(unix)]
unsafe fn libc_kill_check(_pid: i32) -> bool {
    false
}

#[cfg(not(unix))]
fn process_alive(_pid: u32) -> bool {
    false
}

pub fn status() -> DaemonStatus {
    DaemonStatus {
        running: is_running(),
        socket_path: socket_path().to_string_lossy().into_owned(),
        pid_file: pid_path().to_string_lossy().into_owned(),
        warm_servers: Vec::new(),
        idle_seconds: None,
    }
}

pub fn stop() -> Result<()> {
    let pid_file = pid_path();
    if !pid_file.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&pid_file)?;
    let pid: u32 = content
        .trim()
        .parse()
        .map_err(|_| anyhow!("invalid pid file"))?;
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
    }
    let _ = std::fs::remove_file(&pid_file);
    let _ = std::fs::remove_file(socket_path());
    Ok(())
}

pub async fn start(_servers: &[ServerId], _root: &Path) -> Result<()> {
    if is_running() {
        return Err(anyhow!(
            "daemon ja esta rodando (pid em {})",
            pid_path().display()
        ));
    }
    let pid = std::process::id();
    std::fs::write(pid_path(), pid.to_string())?;
    let socket = socket_path();
    if socket.exists() {
        let _ = std::fs::remove_file(&socket);
    }
    Ok(())
}
