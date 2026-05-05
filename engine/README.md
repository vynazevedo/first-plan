# first-plan-engine

Native Rust binary for performance-critical operations in the
[first-plan](https://github.com/vynazevedo/first-plan) Claude Code plugin.

## Why

Plugin operations like building a co-change graph from 50k+ commits or
hashing 10k files for cache invalidation are too slow when done via shell
loops + Claude tokens. This binary handles those operations natively:

| Operation | Shell + Claude | Native engine |
|-----------|----------------|---------------|
| Co-change graph (50k commits) | ~5 min | <2 s |
| Hash 10k files (xxh3) | ~30 s | <500 ms |
| Token cost | ~30k tokens | ~0 |

## Subcommands

### `cochange`

Build the co-change graph from git history.

```
first-plan-engine cochange \
  --repo /path/to/repo \
  --since 180 \
  --min-occurrences 5 \
  --min-ratio 0.5 \
  --output-json /tmp/cc.json
```

### `hash`

Parallel xxh3 hashing of files (cache invalidation).

```
find . -type f | first-plan-engine hash \
  --paths-from-stdin \
  --output-json /tmp/h.json
```

## Install

### Pre-built (recommended)

Download from [Releases](https://github.com/vynazevedo/first-plan/releases) the
binary matching your platform, extract, and place anywhere in your `$PATH`
(or in `${CLAUDE_PLUGIN_ROOT}/engine/bin/` for plugin auto-detection).

### From source

```
git clone https://github.com/vynazevedo/first-plan
cd first-plan/engine
cargo install --path crates/cli
```

Requires Rust 1.75+.

## Architecture

Cargo workspace with two crates:

- `first-plan-core` (lib): git log parsing, co-change matrix, clustering,
  parallel hashing, JSON serialization
- `first-plan-engine` (bin): clap-based CLI exposing core via subcommands

Output is always versioned JSON (`$schema` field) for safe consumption by
plugin skills.

## Build

```
cargo build --release
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

## License

MIT - see [LICENSE](../LICENSE).
