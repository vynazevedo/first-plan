//! first-plan-core - heavy-lifting library for the first-plan plugin.
//!
//! Provides primitives that the CLI (first-plan-engine) exposes as subcommands:
//! - git log parsing (lightweight wrapper over the git CLI)
//! - co-change matrix and cluster detection
//! - parallel xxh3 hashing for cache invalidation
//! - JSON serialization with versioned schemas

pub mod cochange;
pub mod git;
pub mod hash;
pub mod output;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
