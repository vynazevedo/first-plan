# first-plan engine

Rust CLI and library for evidence-backed preparation of AI-assisted code changes.
`fpe` is the primary binary; `first-plan-engine` remains a compatibility alias.

```bash
cargo install --path crates/cli --locked
fpe context --query "validate email" --budget 8000 --json
fpe impact
fpe deployment status
fpe mcp --root /absolute/project/path
```

Source builds require Rust 1.96. Prebuilt Linux, macOS and Windows binaries and
checksums are available in [releases](https://github.com/vynazevedo/first-plan/releases).

The workspace separates reusable analysis in `first-plan-core` from the CLI.
Commands include Git/co-change analysis, symbol indexing/search, LSP, quality,
contracts, evolution, release history, initialization and instruction generation.
ML embeddings and Tree-sitter are optional features. Refer to the
[root README](../README.md) for the complete command reference.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace --locked -- -D warnings
cargo test --workspace --release --locked
```

See [evidence workflow](../docs/evidence-workflow.md),
[evaluations](../docs/evaluations.md) and [release procedure](../docs/releases.md).
