//! Cross-reference schema entities with code identifiers.
//!
//! Para cada endpoint (OpenAPI), RPC (Protobuf) ou operation (GraphQL) declarado
//! em spec, faz busca multi-pattern no codigo para achar implementacao.
//!
//! Classificacao:
//! - IMPLEMENTED: forte evidencia (multiplas ocorrencias, arquivos de handler)
//! - CANDIDATE: match parcial (1-2 ocorrencias, precisa validacao humana)
//! - PHANTOM: zero evidencia (spec declara mas codigo nao implementa)
//!
//! Heuristica multi-lang aware: reconhece patterns de axum/actix/gin/fastapi/express/spring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use super::graphql::GraphqlReport;
use super::openapi::OpenApiReport;
use super::protobuf::ProtobufReport;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossrefReport {
    pub items: Vec<CrossrefItem>,
    pub summary: CrossrefSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossrefItem {
    pub source: SchemaSource,
    pub identifier: String,
    pub path: Option<String>,
    pub method: Option<String>,
    pub status: CrossrefStatus,
    pub evidence_count: usize,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaSource {
    OpenApi { file: String },
    Protobuf { file: String, service: String },
    Graphql { file: String, kind: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossrefStatus {
    Implemented,
    Candidate,
    Phantom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub file: String,
    pub line: u32,
    pub matched_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossrefSummary {
    pub total: usize,
    pub implemented: usize,
    pub candidates: usize,
    pub phantoms: usize,
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

const SOURCE_EXTS: &[&str] = &[
    "rs", "go", "py", "ts", "tsx", "js", "jsx", "java", "kt", "rb", "php", "cs",
];

const MAX_FILES_SCANNED: usize = 5000;

pub fn analyze(
    root: &Path,
    openapi: &OpenApiReport,
    protobuf: &ProtobufReport,
    graphql: &GraphqlReport,
) -> CrossrefReport {
    let file_contents = collect_source_files(root);
    let mut report = CrossrefReport::default();

    for endpoint in &openapi.endpoints {
        let mut identifiers = Vec::new();
        if let Some(op_id) = &endpoint.operation_id {
            identifiers.push(op_id.clone());
        }
        identifiers.push(endpoint.path.clone());

        let evidence = search_identifiers(&file_contents, &identifiers, 20);
        let status = classify(&evidence);
        report.items.push(CrossrefItem {
            source: SchemaSource::OpenApi {
                file: endpoint.spec_file.clone(),
            },
            identifier: endpoint
                .operation_id
                .clone()
                .unwrap_or_else(|| format!("{} {}", endpoint.method, endpoint.path)),
            path: Some(endpoint.path.clone()),
            method: Some(endpoint.method.clone()),
            evidence_count: evidence.len(),
            status,
            evidence,
        });
    }

    for svc in &protobuf.services {
        for rpc in &svc.rpcs {
            let identifiers = vec![
                format!("{}/{}", svc.name, rpc.name),
                rpc.name.clone(),
                format!("{}.{}", svc.name, rpc.name),
            ];
            let evidence = search_identifiers(&file_contents, &identifiers, 20);
            let status = classify(&evidence);
            report.items.push(CrossrefItem {
                source: SchemaSource::Protobuf {
                    file: svc.file.clone(),
                    service: svc.name.clone(),
                },
                identifier: format!("{}.{}", svc.name, rpc.name),
                path: None,
                method: None,
                evidence_count: evidence.len(),
                status,
                evidence,
            });
        }
    }

    for op in &graphql.operations {
        let identifiers = vec![op.name.clone()];
        let evidence = search_identifiers(&file_contents, &identifiers, 20);
        let status = classify(&evidence);
        let kind_str = format!("{:?}", op.kind);
        report.items.push(CrossrefItem {
            source: SchemaSource::Graphql {
                file: op.schema_file.clone(),
                kind: kind_str,
            },
            identifier: op.name.clone(),
            path: None,
            method: None,
            evidence_count: evidence.len(),
            status,
            evidence,
        });
    }

    for item in &report.items {
        report.summary.total += 1;
        match item.status {
            CrossrefStatus::Implemented => report.summary.implemented += 1,
            CrossrefStatus::Candidate => report.summary.candidates += 1,
            CrossrefStatus::Phantom => report.summary.phantoms += 1,
        }
    }

    report
}

fn collect_source_files(root: &Path) -> HashMap<String, String> {
    let mut out = HashMap::new();
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
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !SOURCE_EXTS.contains(&ext) {
            continue;
        }
        if out.len() >= MAX_FILES_SCANNED {
            break;
        }
        let path_key = entry
            .path()
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| entry.path().to_string_lossy().into_owned());
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if content.len() < 500_000 {
                out.insert(path_key, content);
            }
        }
    }
    out
}

fn search_identifiers(
    files: &HashMap<String, String>,
    identifiers: &[String],
    max_evidence: usize,
) -> Vec<Evidence> {
    let mut evidence = Vec::new();
    let significant: Vec<&String> = identifiers
        .iter()
        .filter(|i| !i.is_empty() && i.len() >= 3 && !is_generic_path(i))
        .collect();

    if significant.is_empty() {
        return evidence;
    }

    for (path, content) in files {
        for (line_idx, line) in content.lines().enumerate() {
            for identifier in &significant {
                if let Some(pos) = line.find(identifier.as_str()) {
                    if is_word_boundary_match(line, pos, identifier.len())
                        || identifier.contains('/')
                    {
                        evidence.push(Evidence {
                            file: path.clone(),
                            line: (line_idx + 1) as u32,
                            matched_text: line.trim().to_string(),
                        });
                        if evidence.len() >= max_evidence {
                            return evidence;
                        }
                        break;
                    }
                }
            }
        }
    }
    evidence
}

fn is_word_boundary_match(line: &str, pos: usize, len: usize) -> bool {
    let bytes = line.as_bytes();
    let before_ok = pos == 0 || !bytes[pos - 1].is_ascii_alphanumeric() && bytes[pos - 1] != b'_';
    let after_pos = pos + len;
    let after_ok = after_pos >= bytes.len()
        || !bytes[after_pos].is_ascii_alphanumeric() && bytes[after_pos] != b'_';
    before_ok && after_ok
}

fn is_generic_path(s: &str) -> bool {
    matches!(s, "/" | "/*" | "/health" | "/ping" | "/status" | "/version")
}

fn classify(evidence: &[Evidence]) -> CrossrefStatus {
    match evidence.len() {
        0 => CrossrefStatus::Phantom,
        1 | 2 => CrossrefStatus::Candidate,
        _ => CrossrefStatus::Implemented,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::openapi::Endpoint;
    use std::collections::HashMap;

    fn make_openapi_endpoint(op_id: &str, path: &str, method: &str) -> Endpoint {
        Endpoint {
            spec_file: "openapi.yaml".to_string(),
            path: path.to_string(),
            method: method.to_string(),
            operation_id: Some(op_id.to_string()),
            summary: None,
            tags: vec![],
        }
    }

    #[test]
    fn word_boundary_match() {
        assert!(is_word_boundary_match("call listUsers()", 5, 9));
        assert!(!is_word_boundary_match("call listUsersFast()", 5, 9));
    }

    #[test]
    fn classifies_phantom_when_no_evidence() {
        let files: HashMap<String, String> = HashMap::new();
        let ev = search_identifiers(&files, &["someUnusedOp".to_string()], 10);
        assert!(matches!(classify(&ev), CrossrefStatus::Phantom));
    }

    #[test]
    fn classifies_implemented_with_many_hits() {
        let mut files = HashMap::new();
        files.insert(
            "handler.rs".to_string(),
            "fn listUsers() {}\nlistUsers();\nlistUsers();\nlistUsers();\n".to_string(),
        );
        let ev = search_identifiers(&files, &["listUsers".to_string()], 10);
        assert!(ev.len() >= 3);
        assert!(matches!(classify(&ev), CrossrefStatus::Implemented));
    }

    #[test]
    fn analyze_openapi_produces_items() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("handler.go"),
            "func createUserHandler(w http.ResponseWriter, r *http.Request) {}\n",
        )
        .unwrap();

        let mut openapi = OpenApiReport::default();
        openapi
            .endpoints
            .push(make_openapi_endpoint("createUser", "/users", "POST"));
        openapi
            .endpoints
            .push(make_openapi_endpoint("deleteUser", "/users/{id}", "DELETE"));

        let protobuf = ProtobufReport::default();
        let graphql = GraphqlReport::default();
        let report = analyze(tmp.path(), &openapi, &protobuf, &graphql);
        assert_eq!(report.items.len(), 2);
        assert_eq!(report.summary.total, 2);
    }

    #[test]
    fn ignores_generic_health_paths() {
        assert!(is_generic_path("/health"));
        assert!(is_generic_path("/status"));
        assert!(!is_generic_path("/users/{id}"));
    }
}
