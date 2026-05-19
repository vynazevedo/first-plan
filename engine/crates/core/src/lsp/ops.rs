//! Operacoes LSP: references, definition, documentSymbol, hover, workspaceSymbol.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use super::client::LspClient;
use super::registry::{server_for_path, spec, ServerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub location: Location,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
    Unknown,
}

impl SymbolKind {
    pub fn from_lsp(n: i64) -> Self {
        match n {
            1 => Self::File,
            2 => Self::Module,
            3 => Self::Namespace,
            4 => Self::Package,
            5 => Self::Class,
            6 => Self::Method,
            7 => Self::Property,
            8 => Self::Field,
            9 => Self::Constructor,
            10 => Self::Enum,
            11 => Self::Interface,
            12 => Self::Function,
            13 => Self::Variable,
            14 => Self::Constant,
            15 => Self::String,
            16 => Self::Number,
            17 => Self::Boolean,
            18 => Self::Array,
            19 => Self::Object,
            20 => Self::Key,
            21 => Self::Null,
            22 => Self::EnumMember,
            23 => Self::Struct,
            24 => Self::Event,
            25 => Self::Operator,
            26 => Self::TypeParameter,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    pub content: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpResult {
    References(Vec<Reference>),
    Definitions(Vec<Location>),
    DocumentSymbols(Vec<Symbol>),
    Hover(Option<HoverInfo>),
    WorkspaceSymbols(Vec<Symbol>),
}

pub async fn references(
    client: &LspClient,
    file: &Path,
    line: u32,
    column: u32,
    include_declaration: bool,
) -> Result<Vec<Reference>> {
    open_file(client, file).await?;
    let uri = format!("file://{}", file.display());
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": column },
        "context": { "includeDeclaration": include_declaration },
    });
    let raw = client.request("textDocument/references", params).await?;
    Ok(parse_references(&raw, file))
}

pub async fn definition(
    client: &LspClient,
    file: &Path,
    line: u32,
    column: u32,
) -> Result<Vec<Location>> {
    open_file(client, file).await?;
    let uri = format!("file://{}", file.display());
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": column },
    });
    let raw = client.request("textDocument/definition", params).await?;
    Ok(parse_locations(&raw))
}

pub async fn document_symbol(client: &LspClient, file: &Path) -> Result<Vec<Symbol>> {
    open_file(client, file).await?;
    let uri = format!("file://{}", file.display());
    let params = json!({ "textDocument": { "uri": uri } });
    let raw = client
        .request("textDocument/documentSymbol", params)
        .await?;
    Ok(parse_document_symbols(&raw))
}

pub async fn hover(
    client: &LspClient,
    file: &Path,
    line: u32,
    column: u32,
) -> Result<Option<HoverInfo>> {
    open_file(client, file).await?;
    let uri = format!("file://{}", file.display());
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": column },
    });
    let raw = client.request("textDocument/hover", params).await?;
    Ok(parse_hover(&raw))
}

pub async fn workspace_symbol(client: &LspClient, query: &str) -> Result<Vec<Symbol>> {
    let params = json!({ "query": query });
    let raw = client.request("workspace/symbol", params).await?;
    Ok(parse_workspace_symbols(&raw))
}

async fn open_file(client: &LspClient, file: &Path) -> Result<()> {
    let content = tokio::fs::read_to_string(file).await?;
    let language_id = language_id_for_path(file, client.server_id);
    client.did_open(file, &language_id, &content).await
}

fn language_id_for_path(path: &Path, server: ServerId) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match (server, ext) {
        (ServerId::RustAnalyzer, _) => "rust",
        (ServerId::Gopls, _) => "go",
        (ServerId::Pyright, _) => "python",
        (ServerId::TypeScriptLanguageServer, "tsx") => "typescriptreact",
        (ServerId::TypeScriptLanguageServer, "jsx") => "javascriptreact",
        (ServerId::TypeScriptLanguageServer, "js" | "mjs" | "cjs") => "javascript",
        (ServerId::TypeScriptLanguageServer, _) => "typescript",
        (ServerId::Intelephense, _) => "php",
        (ServerId::Clangd, "c") => "c",
        (ServerId::Clangd, _) => "cpp",
        (ServerId::RubyLsp, _) => "ruby",
        (ServerId::LuaLanguageServer, _) => "lua",
    }
    .to_string()
}

pub fn detect_server_for_file(path: &Path) -> Option<ServerId> {
    server_for_path(path)
}

pub fn server_name(server: ServerId) -> String {
    spec(server).name.to_string()
}

fn parse_location(v: &Value) -> Option<Location> {
    let uri = v.get("uri")?.as_str()?.to_string();
    let range = v.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let file = uri.strip_prefix("file://").unwrap_or(&uri).to_string();
    Some(Location {
        uri,
        file,
        line: start.get("line")?.as_u64()? as u32,
        column: start.get("character")?.as_u64()? as u32,
        end_line: end.get("line")?.as_u64()? as u32,
        end_column: end.get("character")?.as_u64()? as u32,
    })
}

fn parse_locations(raw: &Value) -> Vec<Location> {
    match raw {
        Value::Null => Vec::new(),
        Value::Array(arr) => arr.iter().filter_map(parse_location).collect(),
        v => parse_location(v).map(|x| vec![x]).unwrap_or_default(),
    }
}

fn parse_references(raw: &Value, _file: &Path) -> Vec<Reference> {
    parse_locations(raw)
        .into_iter()
        .map(|location| Reference {
            snippet: extract_snippet(&location),
            location,
        })
        .collect()
}

fn extract_snippet(loc: &Location) -> Option<String> {
    let content = std::fs::read_to_string(&loc.file).ok()?;
    content
        .lines()
        .nth(loc.line as usize)
        .map(|l| l.to_string())
}

fn parse_document_symbols(raw: &Value) -> Vec<Symbol> {
    let mut out = Vec::new();
    if let Some(arr) = raw.as_array() {
        for item in arr {
            collect_doc_symbol(item, None, &mut out);
        }
    }
    out
}

fn collect_doc_symbol(item: &Value, parent: Option<&str>, out: &mut Vec<Symbol>) {
    if let Some(loc) = item.get("location") {
        if let Some(location) = parse_location(loc) {
            let kind = item
                .get("kind")
                .and_then(|k| k.as_i64())
                .map(SymbolKind::from_lsp)
                .unwrap_or(SymbolKind::Unknown);
            out.push(Symbol {
                name: item
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                kind,
                location,
                container: item
                    .get("containerName")
                    .and_then(|c| c.as_str())
                    .map(String::from)
                    .or_else(|| parent.map(String::from)),
                detail: None,
            });
        }
    } else if let Some(range) = item.get("range") {
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let detail = item
            .get("detail")
            .and_then(|d| d.as_str())
            .map(String::from);
        let kind = item
            .get("kind")
            .and_then(|k| k.as_i64())
            .map(SymbolKind::from_lsp)
            .unwrap_or(SymbolKind::Unknown);
        let start = range.get("start").cloned().unwrap_or(Value::Null);
        let end = range.get("end").cloned().unwrap_or(Value::Null);
        let location = Location {
            uri: String::new(),
            file: String::new(),
            line: start.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            column: start.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            end_line: end.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            end_column: end.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        };
        out.push(Symbol {
            name: name.clone(),
            kind,
            location,
            container: parent.map(String::from),
            detail,
        });
        if let Some(children) = item.get("children").and_then(|c| c.as_array()) {
            for child in children {
                collect_doc_symbol(child, Some(&name), out);
            }
        }
    }
}

fn parse_workspace_symbols(raw: &Value) -> Vec<Symbol> {
    let mut out = Vec::new();
    if let Some(arr) = raw.as_array() {
        for item in arr {
            if let Some(loc) = item.get("location") {
                if let Some(location) = parse_location(loc) {
                    let kind = item
                        .get("kind")
                        .and_then(|k| k.as_i64())
                        .map(SymbolKind::from_lsp)
                        .unwrap_or(SymbolKind::Unknown);
                    out.push(Symbol {
                        name: item
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        kind,
                        location,
                        container: item
                            .get("containerName")
                            .and_then(|c| c.as_str())
                            .map(String::from),
                        detail: None,
                    });
                }
            }
        }
    }
    out
}

fn parse_hover(raw: &Value) -> Option<HoverInfo> {
    let contents = raw.get("contents")?;
    let (content, language) = match contents {
        Value::String(s) => (s.clone(), None),
        Value::Object(o) => {
            let kind = o.get("kind").and_then(|k| k.as_str());
            let value = o.get("value").and_then(|v| v.as_str())?;
            (value.to_string(), kind.map(String::from))
        }
        Value::Array(arr) => {
            let parts: Vec<String> = arr
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(o) => o.get("value").and_then(|x| x.as_str()).map(String::from),
                    _ => None,
                })
                .collect();
            (parts.join("\n"), None)
        }
        _ => return None,
    };
    if content.is_empty() {
        None
    } else {
        Some(HoverInfo { content, language })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lsp_location() {
        let v = json!({
            "uri": "file:///x/y.rs",
            "range": {
                "start": { "line": 10, "character": 5 },
                "end":   { "line": 10, "character": 15 }
            }
        });
        let loc = parse_location(&v).unwrap();
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.end_column, 15);
        assert_eq!(loc.file, "/x/y.rs");
    }

    #[test]
    fn parses_hover_string_form() {
        let v = json!({ "contents": "fn foo() -> i32" });
        assert_eq!(parse_hover(&v).unwrap().content, "fn foo() -> i32");
    }

    #[test]
    fn parses_hover_markup_form() {
        let v = json!({
            "contents": { "kind": "markdown", "value": "`fn foo()`" }
        });
        let h = parse_hover(&v).unwrap();
        assert_eq!(h.content, "`fn foo()`");
        assert_eq!(h.language, Some("markdown".into()));
    }
}
