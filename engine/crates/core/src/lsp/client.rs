//! LSP client - JSON-RPC 2.0 sobre stdio.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::registry::{spec, ServerId};

pub struct LspClient {
    pub server_id: ServerId,
    pub root: PathBuf,
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    reader: Arc<Mutex<BufReader<ChildStdout>>>,
    next_id: AtomicI64,
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(60);

impl LspClient {
    pub async fn spawn(server_id: ServerId, root: &Path) -> Result<Self> {
        let spec = spec(server_id);
        let mut cmd = Command::new(spec.binary);
        cmd.args(&spec.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        cmd.kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn {}", spec.binary))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("stdin not captured"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("stdout not captured"))?;

        let client = LspClient {
            server_id,
            root: root.to_path_buf(),
            child,
            stdin: Arc::new(Mutex::new(stdin)),
            reader: Arc::new(Mutex::new(BufReader::new(stdout))),
            next_id: AtomicI64::new(1),
        };

        timeout(INITIALIZE_TIMEOUT, client.initialize())
            .await
            .map_err(|_| anyhow!("LSP initialize timeout"))??;

        Ok(client)
    }

    async fn initialize(&self) -> Result<()> {
        let root_uri = format!("file://{}", self.root.display());
        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "synchronization": { "didSave": true },
                    "references": { "dynamicRegistration": false },
                    "definition": { "dynamicRegistration": false },
                    "documentSymbol": { "dynamicRegistration": false },
                    "hover": { "dynamicRegistration": false, "contentFormat": ["plaintext", "markdown"] },
                },
                "workspace": {
                    "symbol": { "dynamicRegistration": false },
                    "workspaceFolders": true,
                }
            },
            "workspaceFolders": [{
                "uri": root_uri,
                "name": self.root.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "root".into()),
            }],
        });

        let _ = self.request("initialize", params).await?;
        self.notify("initialized", json!({})).await?;
        Ok(())
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_message(&msg).await?;

        timeout(REQUEST_TIMEOUT, self.read_response(id))
            .await
            .map_err(|_| anyhow!("request {} timeout", method))?
    }

    pub async fn notify(&self, method: &str, params: Value) -> Result<()> {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_message(&msg).await
    }

    async fn write_message(&self, msg: &Value) -> Result<()> {
        let body = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(body.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn read_response(&self, want_id: i64) -> Result<Value> {
        loop {
            let msg = self.read_message().await?;
            if let Some(id) = msg.get("id").and_then(|v| v.as_i64()) {
                if id == want_id {
                    if let Some(err) = msg.get("error") {
                        return Err(anyhow!("LSP error: {}", err));
                    }
                    return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
                }
            }
        }
    }

    async fn read_message(&self) -> Result<Value> {
        let mut reader = self.reader.lock().await;
        let mut content_length: Option<usize> = None;

        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("LSP server closed connection"));
            }
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(rest.trim().parse()?);
            }
        }

        let len = content_length.ok_or_else(|| anyhow!("missing Content-Length"))?;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        Ok(serde_json::from_slice(&buf)?)
    }

    pub async fn did_open(&self, file: &Path, language_id: &str, content: &str) -> Result<()> {
        let uri = format!("file://{}", file.display());
        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": language_id,
                "version": 1,
                "text": content,
            }
        });
        self.notify("textDocument/didOpen", params).await
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let _ = self.request("shutdown", Value::Null).await;
        let _ = self.notify("exit", Value::Null).await;
        let _ = self.child.kill().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_framing_roundtrip() {
        let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "test"});
        let body = serde_json::to_string(&msg).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        let full = format!("{}{}", header, body);
        assert!(full.contains("Content-Length"));
        assert!(full.contains("\"method\":\"test\""));
    }
}
