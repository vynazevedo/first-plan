//! Symbol extraction from source files (regex-based, language-aware).
//!
//! Extracts top-level definitions: functions, types, classes, constants. Tree-sitter
//! based extraction lands in v0.5.0; for now regex is fast and good enough as a
//! discoverability index for `/first-plan:reuse` queries.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub language: String,
    pub path: String,
    pub line: u32,
    pub signature: String,
    pub doc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Type,
    Class,
    Constant,
    Method,
}

/// Languages we know how to extract from. Detection is by extension only.
pub fn language_from_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "go" => Some("go"),
        "rs" => Some("rust"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "py" => Some("python"),
        "php" => Some("php"),
        "rb" => Some("ruby"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "swift" => Some("swift"),
        "ex" | "exs" => Some("elixir"),
        _ => None,
    }
}

/// Extract symbols from a source file. Returns empty Vec for unsupported langs.
pub fn extract_symbols(path: &Path, source: &str) -> Result<Vec<Symbol>> {
    let lang = match language_from_path(path) {
        Some(l) => l,
        None => return Ok(Vec::new()),
    };

    let path_str = path.to_string_lossy().into_owned();
    let mut symbols = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        if let Some((kind, name, sig)) = match_line(lang, line) {
            let doc = extract_doc_above(source, idx);
            symbols.push(Symbol {
                name: name.to_string(),
                kind,
                language: lang.to_string(),
                path: path_str.clone(),
                line: line_no,
                signature: sig.to_string(),
                doc,
            });
        }
    }

    Ok(symbols)
}

/// Try to match a single line against language-specific patterns.
/// Returns (kind, name, signature) on hit.
fn match_line<'a>(lang: &str, line: &'a str) -> Option<(SymbolKind, &'a str, &'a str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
        return None;
    }

    match lang {
        "go" => match_go(line),
        "rust" => match_rust(line),
        "typescript" | "javascript" => match_ts(line),
        "python" => match_python(line),
        "php" => match_php(line),
        _ => None,
    }
}

macro_rules! cached_regex {
    ($cell:ident, $pattern:expr) => {
        $cell.get_or_init(|| Regex::new($pattern).expect("invalid regex"))
    };
}

fn match_go(line: &str) -> Option<(SymbolKind, &str, &str)> {
    static FUNC: OnceLock<Regex> = OnceLock::new();
    static TYPE: OnceLock<Regex> = OnceLock::new();
    static CONST: OnceLock<Regex> = OnceLock::new();

    let func_re = cached_regex!(
        FUNC,
        r"^\s*func\s+(?:\([^)]+\)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\("
    );
    let type_re = cached_regex!(
        TYPE,
        r"^\s*type\s+([A-Za-z_][A-Za-z0-9_]*)\s+(?:struct|interface|=)"
    );
    let const_re = cached_regex!(CONST, r"^\s*(?:const|var)\s+([A-Z][A-Za-z0-9_]*)\s+");

    if let Some(c) = func_re.captures(line) {
        // Method has receiver before the name: `func (r T) Name(...)`.
        // Function has no receiver: `func Name(...)`.
        let kind = if line.trim_start().starts_with("func (") {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        let name = c.get(1)?.as_str();
        return Some((kind, name, line.trim()));
    }
    if let Some(c) = type_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = const_re.captures(line) {
        return Some((SymbolKind::Constant, c.get(1)?.as_str(), line.trim()));
    }
    None
}

fn match_rust(line: &str) -> Option<(SymbolKind, &str, &str)> {
    static FN: OnceLock<Regex> = OnceLock::new();
    static STRUCT: OnceLock<Regex> = OnceLock::new();
    static ENUM: OnceLock<Regex> = OnceLock::new();
    static TRAIT: OnceLock<Regex> = OnceLock::new();
    static CONST: OnceLock<Regex> = OnceLock::new();

    let fn_re = cached_regex!(
        FN,
        r"^\s*(?:pub\s+(?:\([^)]*\)\s+)?)?(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"
    );
    let struct_re = cached_regex!(
        STRUCT,
        r"^\s*(?:pub\s+(?:\([^)]*\)\s+)?)?struct\s+([A-Z][A-Za-z0-9_]*)"
    );
    let enum_re = cached_regex!(
        ENUM,
        r"^\s*(?:pub\s+(?:\([^)]*\)\s+)?)?enum\s+([A-Z][A-Za-z0-9_]*)"
    );
    let trait_re = cached_regex!(
        TRAIT,
        r"^\s*(?:pub\s+(?:\([^)]*\)\s+)?)?trait\s+([A-Z][A-Za-z0-9_]*)"
    );
    let const_re = cached_regex!(
        CONST,
        r"^\s*(?:pub\s+(?:\([^)]*\)\s+)?)?(?:const|static)\s+([A-Z_][A-Z0-9_]*)\s*:"
    );

    if let Some(c) = fn_re.captures(line) {
        return Some((SymbolKind::Function, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = struct_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = enum_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = trait_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = const_re.captures(line) {
        return Some((SymbolKind::Constant, c.get(1)?.as_str(), line.trim()));
    }
    None
}

fn match_ts(line: &str) -> Option<(SymbolKind, &str, &str)> {
    static FN: OnceLock<Regex> = OnceLock::new();
    static ARROW: OnceLock<Regex> = OnceLock::new();
    static CLASS: OnceLock<Regex> = OnceLock::new();
    static INTERFACE: OnceLock<Regex> = OnceLock::new();
    static TYPE: OnceLock<Regex> = OnceLock::new();
    static CONST: OnceLock<Regex> = OnceLock::new();

    let fn_re = cached_regex!(
        FN,
        r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][A-Za-z0-9_$]*)"
    );
    let arrow_re = cached_regex!(
        ARROW,
        r"^\s*(?:export\s+)?(?:const|let|var)\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*[:=][^=]*=\s*(?:async\s*)?\("
    );
    let class_re = cached_regex!(
        CLASS,
        r"^\s*(?:export\s+(?:default\s+)?)?(?:abstract\s+)?class\s+([A-Z][A-Za-z0-9_$]*)"
    );
    let interface_re = cached_regex!(
        INTERFACE,
        r"^\s*(?:export\s+)?interface\s+([A-Z][A-Za-z0-9_$]*)"
    );
    let type_re = cached_regex!(TYPE, r"^\s*(?:export\s+)?type\s+([A-Z][A-Za-z0-9_$]*)\s*=");
    let const_re = cached_regex!(
        CONST,
        r"^\s*(?:export\s+)?const\s+([A-Z_][A-Z0-9_]+)\s*[:=]"
    );

    if let Some(c) = fn_re.captures(line) {
        return Some((SymbolKind::Function, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = arrow_re.captures(line) {
        return Some((SymbolKind::Function, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = class_re.captures(line) {
        return Some((SymbolKind::Class, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = interface_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = type_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = const_re.captures(line) {
        return Some((SymbolKind::Constant, c.get(1)?.as_str(), line.trim()));
    }
    None
}

fn match_python(line: &str) -> Option<(SymbolKind, &str, &str)> {
    static DEF: OnceLock<Regex> = OnceLock::new();
    static ASYNC_DEF: OnceLock<Regex> = OnceLock::new();
    static CLASS: OnceLock<Regex> = OnceLock::new();

    let def_re = cached_regex!(DEF, r"^\s*def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(");
    let async_def_re = cached_regex!(
        ASYNC_DEF,
        r"^\s*async\s+def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("
    );
    let class_re = cached_regex!(CLASS, r"^\s*class\s+([A-Z][A-Za-z0-9_]*)");

    if let Some(c) = async_def_re.captures(line) {
        let kind = if line.starts_with("    ") || line.starts_with('\t') {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        return Some((kind, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = def_re.captures(line) {
        let kind = if line.starts_with("    ") || line.starts_with('\t') {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        return Some((kind, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = class_re.captures(line) {
        return Some((SymbolKind::Class, c.get(1)?.as_str(), line.trim()));
    }
    None
}

fn match_php(line: &str) -> Option<(SymbolKind, &str, &str)> {
    static FN: OnceLock<Regex> = OnceLock::new();
    static CLASS: OnceLock<Regex> = OnceLock::new();
    static INTERFACE: OnceLock<Regex> = OnceLock::new();

    let fn_re = cached_regex!(
        FN,
        r"^\s*(?:(?:public|private|protected|static|abstract|final)\s+)*function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("
    );
    let class_re = cached_regex!(
        CLASS,
        r"^\s*(?:abstract\s+|final\s+)?class\s+([A-Z][A-Za-z0-9_]*)"
    );
    let interface_re = cached_regex!(INTERFACE, r"^\s*interface\s+([A-Z][A-Za-z0-9_]*)");

    if let Some(c) = fn_re.captures(line) {
        let kind = if line.starts_with("    ") || line.starts_with('\t') {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        return Some((kind, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = class_re.captures(line) {
        return Some((SymbolKind::Class, c.get(1)?.as_str(), line.trim()));
    }
    if let Some(c) = interface_re.captures(line) {
        return Some((SymbolKind::Type, c.get(1)?.as_str(), line.trim()));
    }
    None
}

/// Extract docstring/comment immediately above a definition line.
/// Looks at the previous lines for `///`, `//`, `#`, `/** */`, `"""..."""`.
fn extract_doc_above(source: &str, def_line_idx: usize) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if def_line_idx == 0 {
        return None;
    }
    let mut doc_lines: Vec<&str> = Vec::new();
    let mut i = def_line_idx as i64 - 1;
    while i >= 0 {
        let line = lines[i as usize].trim();
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("///") {
            doc_lines.push(rest.trim());
        } else if let Some(rest) = line.strip_prefix("//") {
            doc_lines.push(rest.trim());
        } else if let Some(rest) = line.strip_prefix('#') {
            doc_lines.push(rest.trim());
        } else if let Some(rest) = line.strip_prefix("* ") {
            doc_lines.push(rest);
        } else {
            break;
        }
        i -= 1;
    }
    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    Some(doc_lines.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn extracts_go_function() {
        let src =
            "// ValidateEmail validates RFC 5322 emails\nfunc ValidateEmail(s string) error {\n}\n";
        let syms = extract_symbols(&PathBuf::from("foo.go"), src).unwrap();
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "ValidateEmail");
        assert_eq!(syms[0].kind, SymbolKind::Function);
        assert!(syms[0].doc.as_ref().unwrap().contains("RFC 5322"));
    }

    #[test]
    fn extracts_rust_struct() {
        let src = "/// User aggregate root\npub struct User {\n}\n";
        let syms = extract_symbols(&PathBuf::from("foo.rs"), src).unwrap();
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "User");
        assert_eq!(syms[0].kind, SymbolKind::Type);
    }

    #[test]
    fn extracts_typescript_interface() {
        let src = "export interface UserDTO {\n}\n";
        let syms = extract_symbols(&PathBuf::from("foo.ts"), src).unwrap();
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "UserDTO");
        assert_eq!(syms[0].kind, SymbolKind::Type);
    }

    #[test]
    fn extracts_python_class_and_method() {
        let src = "class UserService:\n    def get_user(self, id):\n        pass\n";
        let syms = extract_symbols(&PathBuf::from("foo.py"), src).unwrap();
        assert_eq!(syms.len(), 2);
        assert_eq!(syms[0].name, "UserService");
        assert_eq!(syms[0].kind, SymbolKind::Class);
        assert_eq!(syms[1].name, "get_user");
        assert_eq!(syms[1].kind, SymbolKind::Method);
    }

    #[test]
    fn skips_unsupported_languages() {
        let syms = extract_symbols(&PathBuf::from("foo.unknownext"), "anything").unwrap();
        assert!(syms.is_empty());
    }
}
