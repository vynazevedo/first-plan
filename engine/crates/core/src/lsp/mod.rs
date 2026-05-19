//! LSP integration (v0.6.0).
//!
//! Engine fala com 8 servidores de Language Server Protocol:
//! rust-analyzer, gopls, pyright, typescript-language-server,
//! intelephense, clangd, ruby-lsp, lua-language-server.
//!
//! Layout:
//! - registry: mapeamento linguagem -> server, detecao via PATH e manifests
//! - client: JSON-RPC 2.0 sobre stdio (LSP framing)
//! - ops: implementacao das 5 operacoes (references, definition, documentSymbol, hover, workspaceSymbol)
//! - fallback: tree-sitter / grep quando server nao disponivel
//! - daemon: pool de servers warm (otimizacao para sessoes longas)

pub mod client;
pub mod daemon;
pub mod fallback;
pub mod ops;
pub mod registry;

pub use ops::{
    Location as LspLocation, OpResult, Reference, Symbol as LspSymbol, SymbolKind as LspSymbolKind,
};
pub use registry::{
    detect_all, detect_server, server_for_extension, server_for_path, servers_for_project, spec,
    ServerId, ServerSpec, ServerStatus,
};
