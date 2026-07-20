//! OpenAPI 3.x spec detection and parsing.
//!
//! Suporta:
//! - openapi.yaml, openapi.yml, openapi.json
//! - swagger.yaml, swagger.json (legacy naming - parseia se declara openapi: 3.x)
//! - api-docs.yaml, api-docs.json
//! - Em diretorios comuns: root, docs/, api/, spec/, openapi/
//!
//! Output: lista de endpoints (path + method + operationId + summary + tags).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenApiReport {
    pub specs_found: Vec<SpecFile>,
    pub endpoints: Vec<Endpoint>,
    pub total_endpoints: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFile {
    pub path: String,
    pub title: Option<String>,
    pub version: Option<String>,
    pub openapi_version: Option<String>,
    pub endpoint_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub spec_file: String,
    pub path: String,
    pub method: String,
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub tags: Vec<String>,
}

const CANDIDATE_FILENAMES: &[&str] = &[
    "openapi.yaml",
    "openapi.yml",
    "openapi.json",
    "swagger.yaml",
    "swagger.yml",
    "swagger.json",
    "api-docs.yaml",
    "api-docs.yml",
    "api-docs.json",
];

const CANDIDATE_DIRS: &[&str] = &[
    "",
    "docs",
    "api",
    "spec",
    "openapi",
    "docs/api",
    "public/api",
];

pub fn detect(root: &Path) -> OpenApiReport {
    let mut report = OpenApiReport::default();

    for dir in CANDIDATE_DIRS {
        let search_dir = if dir.is_empty() {
            root.to_path_buf()
        } else {
            root.join(dir)
        };
        if !search_dir.exists() || !search_dir.is_dir() {
            continue;
        }
        for filename in CANDIDATE_FILENAMES {
            let candidate = search_dir.join(filename);
            if candidate.exists() && candidate.is_file() {
                parse_spec(&candidate, root, &mut report);
            }
        }
    }

    report.total_endpoints = report.endpoints.len();
    report
}

fn parse_spec(path: &Path, root: &Path, report: &mut OpenApiReport) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let is_json = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let raw: serde_json::Value = if is_json {
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        }
    } else {
        match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(v) => match serde_json::to_value(v) {
                Ok(j) => j,
                Err(_) => return,
            },
            Err(_) => return,
        }
    };

    let openapi_version = raw
        .get("openapi")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            raw.get("swagger")
                .and_then(|v| v.as_str())
                .map(String::from)
        });

    let title = raw
        .get("info")
        .and_then(|i| i.get("title"))
        .and_then(|t| t.as_str())
        .map(String::from);

    let version = raw
        .get("info")
        .and_then(|i| i.get("version"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    let mut file_endpoint_count = 0;

    if let Some(paths) = raw.get("paths").and_then(|p| p.as_object()) {
        for (path_str, path_item) in paths {
            let Some(item_obj) = path_item.as_object() else {
                continue;
            };
            for method in [
                "get", "post", "put", "delete", "patch", "options", "head", "trace",
            ] {
                let Some(op) = item_obj.get(method) else {
                    continue;
                };
                let operation_id = op
                    .get("operationId")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let summary = op.get("summary").and_then(|v| v.as_str()).map(String::from);
                let tags: Vec<String> = op
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                report.endpoints.push(Endpoint {
                    spec_file: rel.clone(),
                    path: path_str.clone(),
                    method: method.to_uppercase(),
                    operation_id,
                    summary,
                    tags,
                });
                file_endpoint_count += 1;
            }
        }
    }

    report.specs_found.push(SpecFile {
        path: rel,
        title,
        version,
        openapi_version,
        endpoint_count: file_endpoint_count,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_openapi_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("openapi.yaml"),
            r#"
openapi: 3.0.3
info:
  title: Test API
  version: 1.0.0
paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      tags: [users]
    post:
      operationId: createUser
      tags: [users]
  /users/{id}:
    get:
      operationId: getUser
      tags: [users]
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.specs_found.len(), 1);
        assert_eq!(report.endpoints.len(), 3);
        assert!(report
            .endpoints
            .iter()
            .any(|e| e.operation_id.as_deref() == Some("listUsers") && e.method == "GET"));
    }

    #[test]
    fn parses_openapi_json() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("openapi.json"),
            r#"{
              "openapi": "3.0.0",
              "info": {"title": "T", "version": "1"},
              "paths": {
                "/health": {
                  "get": {"operationId": "healthCheck"}
                }
              }
            }"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.endpoints.len(), 1);
        assert_eq!(
            report.endpoints[0].operation_id.as_deref(),
            Some("healthCheck")
        );
    }

    #[test]
    fn finds_spec_in_docs_dir() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        fs::write(
            tmp.path().join("docs/openapi.yaml"),
            "openapi: 3.0.0\ninfo: {title: X, version: '1'}\npaths: {/x: {get: {}}}\n",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.specs_found.len(), 1);
    }

    #[test]
    fn empty_when_no_spec() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.specs_found.len(), 0);
        assert_eq!(report.endpoints.len(), 0);
    }
}
