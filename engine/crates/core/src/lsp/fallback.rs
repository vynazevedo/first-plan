//! Fallbacks usados quando o LSP server nao esta instalado.
//!
//! Cadeia: LSP -> tree-sitter (quando feature ativa) -> BM25/grep.
//! Estes fallbacks usam apenas o que esta sempre disponivel (grep).

use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

use super::ops::{Location, Reference};

pub fn references_via_grep(root: &Path, identifier: &str) -> Result<Vec<Reference>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if is_excluded(entry.path()) {
            continue;
        }
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (idx, line) in content.lines().enumerate() {
            let mut start = 0usize;
            while let Some(pos) = line[start..].find(identifier) {
                let abs = start + pos;
                let prev_ok = abs == 0
                    || !line.as_bytes()[abs - 1].is_ascii_alphanumeric()
                        && line.as_bytes()[abs - 1] != b'_';
                let after = abs + identifier.len();
                let next_ok = after >= line.len()
                    || (!line.as_bytes()[after].is_ascii_alphanumeric()
                        && line.as_bytes()[after] != b'_');
                if prev_ok && next_ok {
                    let path_str = entry.path().to_string_lossy().into_owned();
                    out.push(Reference {
                        location: Location {
                            uri: format!("file://{}", path_str),
                            file: path_str,
                            line: idx as u32,
                            column: abs as u32,
                            end_line: idx as u32,
                            end_column: (abs + identifier.len()) as u32,
                        },
                        snippet: Some(line.to_string()),
                    });
                }
                start += pos + identifier.len();
            }
        }
    }
    Ok(out)
}

pub fn workspace_symbols_via_grep(
    root: &Path,
    query: &str,
) -> Result<Vec<crate::lsp::ops::Symbol>> {
    use crate::lsp::ops::{Symbol, SymbolKind};
    let mut out = Vec::new();
    let needle = query.to_lowercase();
    let patterns = [
        ("fn ", SymbolKind::Function),
        ("func ", SymbolKind::Function),
        ("def ", SymbolKind::Function),
        ("function ", SymbolKind::Function),
        ("class ", SymbolKind::Class),
        ("struct ", SymbolKind::Struct),
        ("interface ", SymbolKind::Interface),
        ("type ", SymbolKind::TypeParameter),
        ("enum ", SymbolKind::Enum),
        ("const ", SymbolKind::Constant),
    ];

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .take(20000)
    {
        if is_excluded(entry.path()) {
            continue;
        }
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (idx, line) in content.lines().enumerate() {
            for (kw, kind) in &patterns {
                if let Some(pos) = line.find(kw) {
                    let after = &line[pos + kw.len()..];
                    let name: String = after
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if name.is_empty() {
                        continue;
                    }
                    if !name.to_lowercase().contains(&needle) {
                        continue;
                    }
                    let path_str = entry.path().to_string_lossy().into_owned();
                    out.push(Symbol {
                        name,
                        kind: kind.clone(),
                        location: Location {
                            uri: format!("file://{}", path_str),
                            file: path_str,
                            line: idx as u32,
                            column: pos as u32,
                            end_line: idx as u32,
                            end_column: line.len() as u32,
                        },
                        container: None,
                        detail: Some(line.trim().to_string()),
                    });
                    if out.len() >= 200 {
                        return Ok(out);
                    }
                }
            }
        }
    }
    Ok(out)
}

pub fn hover_via_source(file: &Path, line: u32) -> Result<Option<String>> {
    let content = std::fs::read_to_string(file)?;
    let lines: Vec<&str> = content.lines().collect();
    let idx = line as usize;
    if idx >= lines.len() {
        return Ok(None);
    }
    let start = idx.saturating_sub(2);
    let end = (idx + 3).min(lines.len());
    let snippet = lines[start..end].join("\n");
    Ok(Some(snippet))
}

fn is_excluded(p: &Path) -> bool {
    let s = p.to_string_lossy();
    s.contains("/target/")
        || s.contains("/node_modules/")
        || s.contains("/vendor/")
        || s.contains("/.git/")
        || s.contains("/dist/")
        || s.contains("/build/")
        || s.contains("/.first-plan/")
        || s.ends_with(".min.js")
        || s.ends_with(".lock")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn grep_finds_word_boundary_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let mut f = fs::File::create(tmp.path().join("a.rs")).unwrap();
        writeln!(f, "fn foo() {{}}").unwrap();
        writeln!(f, "let x = foobar;").unwrap();
        writeln!(f, "foo();").unwrap();
        drop(f);

        let refs = references_via_grep(tmp.path(), "foo").unwrap();
        assert_eq!(
            refs.len(),
            2,
            "deve achar 2 matches exatos, n {} (matches em foobar)",
            refs.len()
        );
    }

    #[test]
    fn workspace_symbol_grep_finds_fn_def() {
        let tmp = tempfile::tempdir().unwrap();
        let mut f = fs::File::create(tmp.path().join("lib.rs")).unwrap();
        writeln!(f, "pub fn hello_world() {{}}").unwrap();
        writeln!(f, "struct Config {{}}").unwrap();
        drop(f);

        let syms = workspace_symbols_via_grep(tmp.path(), "hello").unwrap();
        assert!(syms.iter().any(|s| s.name == "hello_world"));
    }
}
