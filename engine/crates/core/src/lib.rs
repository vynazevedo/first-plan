//! first-plan-core - heavy-lifting library for the first-plan plugin.
//!
//! Provides primitives that the CLI (first-plan-engine) exposes as subcommands:
//! - git log parsing (lightweight wrapper over the git CLI)
//! - co-change matrix and cluster detection
//! - parallel xxh3 hashing for cache invalidation
//! - symbol extraction and BM25-based semantic search (v0.4.0+)
//! - JSON serialization with versioned schemas

pub mod ast;
pub mod cochange;
pub mod embeddings;
pub mod git;
pub mod hash;
pub mod index;
pub mod output;
pub mod search;
pub mod symbols;
pub mod tokenize;

/// Whether this binary was built with the `ml` feature enabled.
pub const ML_ENABLED: bool = cfg!(feature = "ml");

/// Whether this binary was built with the `tree-sitter` feature enabled.
pub const AST_ENABLED: bool = cfg!(feature = "tree-sitter");

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
