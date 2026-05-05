//! BM25 search over the SQLite-backed symbol index.
//!
//! BM25 formula (Okapi):
//!   score(D, Q) = sum_{q in Q} IDF(q) * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * |D| / avgdl))
//!
//! Defaults: k1 = 1.5, b = 0.75. These are standard.

use crate::symbols::{Symbol, SymbolKind};
use crate::tokenize::tokenize;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const K1: f64 = 1.5;
const B: f64 = 0.75;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchHit {
    pub symbol: Symbol,
    pub score: f64,
    pub matched_tokens: Vec<String>,
}

/// Search the index, returning top-N hits ranked by BM25.
pub fn search(db_path: &Path, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
    let conn = Connection::open(db_path).with_context(|| format!("open {}", db_path.display()))?;

    let tokens = tokenize(query);
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let total_docs: f64 = conn
        .query_row("SELECT value FROM meta WHERE key = 'total_docs'", [], |r| {
            r.get::<_, String>(0)
        })?
        .parse()
        .unwrap_or(0.0);

    let avg_doc_length: f64 = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'avg_doc_length'",
            [],
            |r| r.get::<_, String>(0),
        )?
        .parse()
        .unwrap_or(1.0);

    if total_docs == 0.0 {
        return Ok(Vec::new());
    }

    // For each query token: count documents containing it (df), and accumulate
    // BM25 contributions per symbol.
    let mut scores: HashMap<i64, (f64, Vec<String>)> = HashMap::new();

    for token in &tokens {
        let df: f64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT symbol_id) FROM tokens WHERE token = ?1",
                params![token],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0) as f64;

        if df == 0.0 {
            continue;
        }

        let idf = ((total_docs - df + 0.5) / (df + 0.5) + 1.0).ln();

        let mut stmt = conn.prepare(
            "SELECT t.symbol_id, t.tf, s.doc_length
             FROM tokens t
             JOIN symbols s ON s.id = t.symbol_id
             WHERE t.token = ?1",
        )?;

        let rows = stmt.query_map(params![token], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)? as f64,
                row.get::<_, i64>(2)? as f64,
            ))
        })?;

        for row in rows.flatten() {
            let (symbol_id, tf, doc_length) = row;
            let denom = tf + K1 * (1.0 - B + B * doc_length / avg_doc_length);
            let contribution = idf * (tf * (K1 + 1.0)) / denom;
            let entry = scores.entry(symbol_id).or_insert_with(|| (0.0, Vec::new()));
            entry.0 += contribution;
            entry.1.push(token.clone());
        }
    }

    // Pick top-N by score
    let mut ranked: Vec<(i64, f64, Vec<String>)> = scores
        .into_iter()
        .map(|(id, (score, matched))| (id, score, matched))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);

    // Hydrate symbols
    let mut hits = Vec::with_capacity(ranked.len());
    for (symbol_id, score, matched) in ranked {
        let symbol = load_symbol(&conn, symbol_id)?;
        hits.push(SearchHit {
            symbol,
            score,
            matched_tokens: matched,
        });
    }
    Ok(hits)
}

fn load_symbol(conn: &Connection, id: i64) -> Result<Symbol> {
    let (name, kind_str, language, path, line, signature, doc): (
        String,
        String,
        String,
        String,
        i64,
        String,
        Option<String>,
    ) = conn.query_row(
        "SELECT name, kind, language, path, line, signature, doc FROM symbols WHERE id = ?1",
        params![id],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        },
    )?;

    let kind = match kind_str.as_str() {
        "function" => SymbolKind::Function,
        "type" => SymbolKind::Type,
        "class" => SymbolKind::Class,
        "constant" => SymbolKind::Constant,
        "method" => SymbolKind::Method,
        _ => SymbolKind::Function,
    };

    Ok(Symbol {
        name,
        kind,
        language,
        path,
        line: line as u32,
        signature,
        doc,
    })
}

/// Search modes available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Pure BM25 over tokenized identifiers.
    Bm25,
    /// Pure cosine similarity over embeddings (requires --features=ml index).
    Embed,
    /// Hybrid: alpha * BM25_normalized + (1-alpha) * cosine_normalized.
    Hybrid,
}

/// Hybrid search combining BM25 and cosine similarity.
///
/// `alpha` controls the BM25 weight; `1.0 - alpha` is the embedding weight.
/// Default 0.3 favors embeddings (semantic) when available, but keeps BM25
/// signal for exact matches.
///
/// Returns top-K hits ranked by the combined score.
pub fn search_hybrid(
    db_path: &Path,
    query: &str,
    query_embedding: &[f32],
    limit: usize,
    alpha: f32,
) -> Result<Vec<SearchHit>> {
    let conn = Connection::open(db_path).with_context(|| format!("open {}", db_path.display()))?;

    // Step 1: BM25 hits with raw scores (we'll normalize)
    let bm25_hits = search(db_path, query, 200)?; // grab more to mix
    let max_bm25 = bm25_hits
        .iter()
        .map(|h| h.score)
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    // Step 2: cosine similarity over all symbols with embedding
    let mut stmt = conn.prepare("SELECT id, embedding FROM symbols WHERE embedding IS NOT NULL")?;
    let mut combined: HashMap<i64, (f64, Vec<String>)> = HashMap::new();

    // Seed with BM25 scores normalized to [0, 1]
    for hit in &bm25_hits {
        if let Some(symbol_id) = symbol_id_for_path_line(&conn, &hit.symbol.path, hit.symbol.line)?
        {
            let normalized = (hit.score / max_bm25).min(1.0);
            combined.insert(
                symbol_id,
                (alpha as f64 * normalized, hit.matched_tokens.clone()),
            );
        }
    }

    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        Ok((id, bytes))
    })?;

    for row in rows.flatten() {
        let (symbol_id, bytes) = row;
        let symbol_emb = crate::embeddings::bytes_to_f32_vec(&bytes);
        if symbol_emb.len() != query_embedding.len() {
            continue;
        }
        let cosine = crate::embeddings::cosine_similarity(query_embedding, &symbol_emb);
        // Map cosine [-1, 1] to [0, 1]
        let normalized = ((cosine + 1.0) / 2.0).clamp(0.0, 1.0) as f64;
        let weight = (1.0 - alpha) as f64;
        let entry = combined
            .entry(symbol_id)
            .or_insert_with(|| (0.0, Vec::new()));
        entry.0 += weight * normalized;
    }

    // Top-K by combined score
    let mut ranked: Vec<(i64, f64, Vec<String>)> = combined
        .into_iter()
        .map(|(id, (score, matched))| (id, score, matched))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);

    let mut hits = Vec::with_capacity(ranked.len());
    for (symbol_id, score, matched) in ranked {
        let symbol = load_symbol(&conn, symbol_id)?;
        hits.push(SearchHit {
            symbol,
            score,
            matched_tokens: matched,
        });
    }
    Ok(hits)
}

fn symbol_id_for_path_line(conn: &Connection, path: &str, line: u32) -> Result<Option<i64>> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM symbols WHERE path = ?1 AND line = ?2 LIMIT 1",
            params![path, line as i64],
            |r| r.get(0),
        )
        .optional()?;
    Ok(id)
}

/// Pure embedding search (cosine similarity only, no BM25).
pub fn search_embed(
    db_path: &Path,
    query_embedding: &[f32],
    limit: usize,
) -> Result<Vec<SearchHit>> {
    let conn = Connection::open(db_path).with_context(|| format!("open {}", db_path.display()))?;

    let mut stmt = conn.prepare("SELECT id, embedding FROM symbols WHERE embedding IS NOT NULL")?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        Ok((id, bytes))
    })?;

    let mut scored: Vec<(i64, f64)> = rows
        .flatten()
        .filter_map(|(id, bytes)| {
            let emb = crate::embeddings::bytes_to_f32_vec(&bytes);
            if emb.len() != query_embedding.len() {
                return None;
            }
            let cosine = crate::embeddings::cosine_similarity(query_embedding, &emb);
            Some((id, cosine as f64))
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let mut hits = Vec::with_capacity(scored.len());
    for (id, score) in scored {
        let symbol = load_symbol(&conn, id)?;
        hits.push(SearchHit {
            symbol,
            score,
            matched_tokens: Vec::new(),
        });
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::build_index;
    use tempfile::TempDir;

    fn fixture_symbols() -> Vec<Symbol> {
        vec![
            Symbol {
                name: "ValidateEmail".into(),
                kind: SymbolKind::Function,
                language: "go".into(),
                path: "pkg/validation/email.go".into(),
                line: 12,
                signature: "func ValidateEmail(s string) error".into(),
                doc: Some("Validate RFC 5322 email format".into()),
            },
            Symbol {
                name: "FormatCurrency".into(),
                kind: SymbolKind::Function,
                language: "go".into(),
                path: "pkg/format/currency.go".into(),
                line: 5,
                signature: "func FormatCurrency(amount int64) string".into(),
                doc: Some("Format integer cents as BRL string".into()),
            },
            Symbol {
                name: "User".into(),
                kind: SymbolKind::Type,
                language: "go".into(),
                path: "internal/domain/user.go".into(),
                line: 3,
                signature: "type User struct".into(),
                doc: None,
            },
        ]
    }

    fn build_test_index() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let db = tmp.path().join("idx.db");
        build_index(&db, &fixture_symbols()).unwrap();
        (tmp, db)
    }

    #[test]
    fn finds_by_name_token() {
        let (_tmp, db) = build_test_index();
        let hits = search(&db, "validate email", 5).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].symbol.name, "ValidateEmail");
    }

    #[test]
    fn finds_via_doc_terms() {
        let (_tmp, db) = build_test_index();
        let hits = search(&db, "RFC 5322", 5).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].symbol.name, "ValidateEmail");
    }

    #[test]
    fn semantic_via_shared_vocab() {
        let (_tmp, db) = build_test_index();
        // Query usa "currency" - encontra FormatCurrency mesmo sem nome exato
        let hits = search(&db, "currency formatting", 5).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].symbol.name, "FormatCurrency");
    }

    #[test]
    fn ranks_relevant_higher() {
        let (_tmp, db) = build_test_index();
        let hits = search(&db, "email validation rfc", 5).unwrap();
        assert_eq!(hits[0].symbol.name, "ValidateEmail");
    }

    #[test]
    fn empty_query_returns_empty() {
        let (_tmp, db) = build_test_index();
        let hits = search(&db, "", 5).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn no_matches_returns_empty() {
        let (_tmp, db) = build_test_index();
        let hits = search(&db, "kubernetes deployment", 5).unwrap();
        assert!(hits.is_empty());
    }
}
