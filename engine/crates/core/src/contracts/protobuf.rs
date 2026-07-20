//! Protobuf spec detection and parsing.
//!
//! Regex-based parser leve - evita dependencia no protoc binary (que exigiria
//! setup complexo em CI e ambientes air-gapped).
//!
//! Extrai: package, service names, rpc method names, message names.
//! Nao tenta type-check completo - suficiente pra crossref no codigo.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtobufReport {
    pub files_found: Vec<ProtoFile>,
    pub services: Vec<ProtoService>,
    pub total_services: usize,
    pub total_rpcs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoFile {
    pub path: String,
    pub package: Option<String>,
    pub service_count: usize,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoService {
    pub file: String,
    pub package: Option<String>,
    pub name: String,
    pub rpcs: Vec<RpcMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethod {
    pub name: String,
    pub request_type: String,
    pub response_type: String,
    pub streaming: Streaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Streaming {
    Unary,
    ClientStream,
    ServerStream,
    Bidi,
}

const EXCLUDED_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "vendor",
    ".git",
    "dist",
    "build",
    ".first-plan",
];

pub fn detect(root: &Path) -> ProtobufReport {
    let mut report = ProtobufReport::default();

    let files = find_proto_files(root);
    for path in &files {
        parse_proto(path, root, &mut report);
    }

    report.total_services = report.services.len();
    report.total_rpcs = report.services.iter().map(|s| s.rpcs.len()).sum();
    report
}

fn find_proto_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(10)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|d| name == *d)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("proto"))
            .unwrap_or(false)
        {
            out.push(entry.path().to_path_buf());
        }
    }
    out
}

fn parse_proto(path: &Path, root: &Path, report: &mut ProtobufReport) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    let package = extract_package(&content);
    let services = extract_services(&content);
    let message_count = extract_message_count(&content);

    report.files_found.push(ProtoFile {
        path: rel.clone(),
        package: package.clone(),
        service_count: services.len(),
        message_count,
    });

    for svc in services {
        report.services.push(ProtoService {
            file: rel.clone(),
            package: package.clone(),
            name: svc.0,
            rpcs: svc.1,
        });
    }
}

fn extract_package(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("package ") {
            let name = rest.trim_end_matches(';').trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn extract_services(content: &str) -> Vec<(String, Vec<RpcMethod>)> {
    let mut out = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("service ") {
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('{')
                .to_string();
            if name.is_empty() {
                continue;
            }
            let mut rpcs = Vec::new();
            let mut depth = if trimmed.ends_with('{') { 1 } else { 0 };

            for body_line in lines.by_ref() {
                let bt = body_line.trim();
                if bt.contains('{') {
                    depth += bt.matches('{').count() as i32;
                }
                if bt.contains('}') {
                    depth -= bt.matches('}').count() as i32;
                    if depth <= 0 {
                        break;
                    }
                }
                if let Some(rpc) = parse_rpc_line(bt) {
                    rpcs.push(rpc);
                }
            }
            out.push((name, rpcs));
        }
    }
    out
}

fn parse_rpc_line(line: &str) -> Option<RpcMethod> {
    let line = line.trim();
    let rest = line.strip_prefix("rpc ")?;
    let name_end = rest.find('(')?;
    let name = rest[..name_end].trim().to_string();

    let after_open = &rest[name_end + 1..];
    let close = after_open.find(')')?;
    let req_raw = after_open[..close].trim();
    let (client_stream, req_type) = if let Some(r) = req_raw.strip_prefix("stream ") {
        (true, r.trim().to_string())
    } else {
        (false, req_raw.to_string())
    };

    let after_first_close = &after_open[close + 1..];
    let returns_idx = after_first_close.find("returns")?;
    let returns_body = &after_first_close[returns_idx + "returns".len()..];
    let open2 = returns_body.find('(')?;
    let after_open2 = &returns_body[open2 + 1..];
    let close2 = after_open2.find(')')?;
    let resp_raw = after_open2[..close2].trim();
    let (server_stream, resp_type) = if let Some(r) = resp_raw.strip_prefix("stream ") {
        (true, r.trim().to_string())
    } else {
        (false, resp_raw.to_string())
    };

    let streaming = match (client_stream, server_stream) {
        (false, false) => Streaming::Unary,
        (true, false) => Streaming::ClientStream,
        (false, true) => Streaming::ServerStream,
        (true, true) => Streaming::Bidi,
    };

    Some(RpcMethod {
        name,
        request_type: req_type,
        response_type: resp_type,
        streaming,
    })
}

fn extract_message_count(content: &str) -> usize {
    let mut count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("message ") {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_simple_service() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("greeter.proto"),
            r#"
syntax = "proto3";

package myapp.greeter.v1;

service Greeter {
  rpc SayHello(HelloRequest) returns (HelloReply);
  rpc StreamHellos(HelloRequest) returns (stream HelloReply);
}

message HelloRequest {
  string name = 1;
}

message HelloReply {
  string message = 1;
}
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.files_found.len(), 1);
        assert_eq!(report.services.len(), 1);
        let svc = &report.services[0];
        assert_eq!(svc.name, "Greeter");
        assert_eq!(svc.package.as_deref(), Some("myapp.greeter.v1"));
        assert_eq!(svc.rpcs.len(), 2);
        assert_eq!(svc.rpcs[0].name, "SayHello");
        assert!(matches!(svc.rpcs[0].streaming, Streaming::Unary));
        assert!(matches!(svc.rpcs[1].streaming, Streaming::ServerStream));
    }

    #[test]
    fn ignores_excluded_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("node_modules/foo")).unwrap();
        fs::write(
            tmp.path().join("node_modules/foo/x.proto"),
            "service X { rpc Y(R) returns (R); }",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.files_found.len(), 0);
    }

    #[test]
    fn empty_when_no_proto() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.services.len(), 0);
    }
}
