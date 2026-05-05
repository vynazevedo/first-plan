---
name: first-plan-lens-rust
description: Stack lens para Rust. Use durante Discovery quando Cargo.toml for detectado. Cobre binários, libs, async runtimes (tokio/async-std), error handling, web frameworks (axum, actix-web, rocket).
version: 0.1.0
---

# Lens Rust

## Detecção fina

| Sinal | Variante |
|-------|----------|
| `[[bin]]` em Cargo.toml ou `src/main.rs` | Binário |
| `[lib]` em Cargo.toml ou `src/lib.rs` | Lib |
| `axum` / `actix-web` / `rocket` / `warp` em deps | HTTP API |
| `tonic` em deps | gRPC |
| `tokio` em deps | Async runtime tokio |
| `async-std` em deps | Async runtime async-std |
| `clap` em deps + bin | CLI |
| `tauri` em deps | Tauri (desktop) |
| `bevy` | Game/3D |
| `[workspace]` em Cargo.toml | Workspace (monorepo Rust) |

## Extração de padrões

### Estrutura

- `src/main.rs` ou `src/bin/<name>.rs`
- `src/lib.rs` + módulos
- Workspaces - quais crates, dependências entre eles
- `tests/` integration tests
- `examples/` exemplos
- `benches/` benchmarks

### Error handling

- `thiserror` (typed errors)
- `anyhow` (boxed errors em apps)
- `?` operator usage
- Custom Error enums

### Async

- Runtime: tokio vs async-std vs smol
- `Send + Sync` constraints
- Spawning tasks (`tokio::spawn`)
- Channels (`tokio::sync::mpsc`, `crossbeam`)

### Macros

- `derive` macros customizados (sinal: dependências de proc-macro crates)
- `macro_rules!` definidos no projeto

### Testing

- `#[test]` unit tests inline
- `tests/` integration
- `criterion` para benchmarks
- `proptest`, `quickcheck` (property testing)
- `mockall` (mocks)
- `wiremock` (HTTP mocks)

### Web

axum:
- Handlers como funções `async fn(...) -> impl IntoResponse`
- Extractors (`State`, `Json`, `Path`, `Query`)
- Routers compostos

actix-web:
- Handlers com `web::Data`, `web::Json`
- App factory

### Logging

- `tracing` ecosystem (tracing + tracing-subscriber + tracing-opentelemetry)
- `log` crate + env_logger (legacy)

### Build / config

- `cargo-make`, `just` em vez de Makefile
- `clippy.toml`, `rustfmt.toml`
- Edition (2018, 2021, 2024)
- `MSRV` (minimum supported Rust version)

## Output

Padrão. Atenção:
- `02-conventions/errors.md` - thiserror vs anyhow vs custom
- `02-conventions/di.md` - Rust geralmente usa State/Context structs em vez de DI clássico

## Confidence rules

Aumentar:
- `clippy.toml` strict + zero warnings nos commits recentes
- Padrão consistente de error handling

Reduzir:
- `unsafe` blocks em código não-justificado por SAFETY: comment
- `unwrap()` / `expect()` em produção sem justificativa

## Anti-padrões comuns

- `unwrap()` sem `expect("razão")` em produção
- `clone()` excessivo (smell de fight com borrow checker)
- `Arc<Mutex<T>>` quando `RwLock` ou channel resolveria melhor
- `panic!` em lib (deveria retornar Result)
