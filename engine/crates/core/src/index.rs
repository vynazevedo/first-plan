//! Index symbols into a SQLite-backed BM25-friendly store.
//!
//! Schema:
//! - `symbols`: every symbol with name, kind, language, path, line, signature, doc
//! - `tokens`: inverted index (token -> symbol_id, term_frequency)
//! - `meta`: corpus statistics (total docs, average doc length)
//!
//! BM25 scoring happens in `search.rs`. This module is the indexer side.

use crate::symbols::{extract_symbols, language_from_path, Symbol};
use crate::tokenize::tokenize;
use anyhow::{Context, Result};
use rayon::prelude::*;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use walkdir::WalkDir;

/// Default ignore patterns for indexing.
fn default_ignores() -> Vec<&'static str> {
    vec![
        ".git",
        "node_modules",
        "vendor",
        "target",
        "dist",
        "build",
        ".next",
        ".nuxt",
        "venv",
        ".venv",
        "__pycache__",
        ".cache",
        "coverage",
        ".first-plan/cache",
    ]
}

fn should_skip(path: &Path, ignores: &[&str]) -> bool {
    let s = path.to_string_lossy();
    ignores.iter().any(|p| s.contains(p))
}

/// Walk the project tree, extract symbols from supported source files.
pub fn collect_symbols(root: &Path) -> Result<Vec<Symbol>> {
    let ignores = default_ignores();
    let files: Vec<PathBuf> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !should_skip(e.path(), &ignores))
        .filter(|e| language_from_path(e.path()).is_some())
        .map(|e| e.into_path())
        .collect();

    let symbols: Vec<Symbol> = files
        .par_iter()
        .flat_map(|path| {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|content| extract_symbols(path, &content).ok())
                .unwrap_or_default()
        })
        .collect();

    Ok(symbols)
}

/// Build a fresh SQLite index from a list of symbols.
pub fn build_index(db_path: &Path, symbols: &[Symbol]) -> Result<IndexStats> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    if db_path.exists() {
        std::fs::remove_file(db_path).ok();
    }

    let conn = Mutex::new(
        Connection::open(db_path).with_context(|| format!("open {}", db_path.display()))?,
    );

    {
        let conn = conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE symbols (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                language TEXT NOT NULL,
                path TEXT NOT NULL,
                line INTEGER NOT NULL,
                signature TEXT NOT NULL,
                doc TEXT,
                doc_length INTEGER NOT NULL
            );
            CREATE TABLE tokens (
                token TEXT NOT NULL,
                symbol_id INTEGER NOT NULL,
                tf INTEGER NOT NULL,
                PRIMARY KEY (token, symbol_id)
            );
            CREATE INDEX idx_tokens_token ON tokens(token);
            CREATE INDEX idx_tokens_symbol ON tokens(symbol_id);
            CREATE TABLE meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
    }

    let mut total_doc_length: u64 = 0;

    {
        let mut conn = conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut insert_symbol = tx.prepare(
                "INSERT INTO symbols (id, name, kind, language, path, line, signature, doc, doc_length)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;
            let mut insert_token = tx.prepare(
                "INSERT INTO tokens (token, symbol_id, tf) VALUES (?1, ?2, ?3)
                 ON CONFLICT(token, symbol_id) DO UPDATE SET tf = tf + excluded.tf",
            )?;

            for (idx, sym) in symbols.iter().enumerate() {
                let id = (idx + 1) as i64;
                let kind = serde_json::to_string(&sym.kind)?
                    .trim_matches('"')
                    .to_string();
                let mut tokens = tokenize(&sym.name);
                tokens.extend(tokenize(&sym.path));
                if let Some(d) = &sym.doc {
                    tokens.extend(tokenize(d));
                }
                let doc_length = tokens.len() as u64;
                total_doc_length += doc_length;

                insert_symbol.execute(params![
                    id,
                    sym.name,
                    kind,
                    sym.language,
                    sym.path,
                    sym.line,
                    sym.signature,
                    sym.doc,
                    doc_length as i64,
                ])?;

                let mut tf_map: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                for t in tokens {
                    *tf_map.entry(t).or_insert(0) += 1;
                }
                for (token, tf) in tf_map {
                    insert_token.execute(params![token, id, tf])?;
                }
            }
        }
        tx.commit()?;
    }

    let total_docs = symbols.len() as u64;
    let avg_doc_length = if total_docs > 0 {
        total_doc_length as f64 / total_docs as f64
    } else {
        0.0
    };

    {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO meta (key, value) VALUES ('total_docs', ?1)",
            params![total_docs.to_string()],
        )?;
        conn.execute(
            "INSERT INTO meta (key, value) VALUES ('avg_doc_length', ?1)",
            params![avg_doc_length.to_string()],
        )?;
    }

    Ok(IndexStats {
        total_symbols: total_docs as u32,
        total_doc_length: total_doc_length as u32,
        avg_doc_length,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct IndexStats {
    pub total_symbols: u32,
    pub total_doc_length: u32,
    pub avg_doc_length: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn collects_symbols_from_directory() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("a.go"),
            "func ValidateEmail(s string) error { return nil }\n",
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("b.py"),
            "class UserService:\n    def get_user(self, id): pass\n",
        )
        .unwrap();
        let syms = collect_symbols(tmp.path()).unwrap();
        assert!(syms.len() >= 3);
    }

    #[test]
    fn builds_index_with_correct_schema() {
        let tmp = TempDir::new().unwrap();
        let symbols = vec![Symbol {
            name: "ValidateEmail".into(),
            kind: crate::symbols::SymbolKind::Function,
            language: "go".into(),
            path: "foo.go".into(),
            line: 1,
            signature: "func ValidateEmail(s string) error".into(),
            doc: Some("Validates RFC 5322".into()),
        }];

        let db_path = tmp.path().join("idx.db");
        let stats = build_index(&db_path, &symbols).unwrap();
        assert_eq!(stats.total_symbols, 1);

        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let token_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tokens WHERE symbol_id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(token_count > 0);
    }
}
