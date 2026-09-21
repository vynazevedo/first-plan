# Release procedure

Use SemVer. Document behavior changes and incomplete capabilities explicitly.
Update the workspace, the two local packages in Cargo.lock, plugin and marketplace
versions, README badges and the dated CHANGELOG entry together.

```bash
python3 tools/check_release.py --tag v1.5.0
python3 tools/validate_frontmatter.py
cd engine
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --release --locked
cargo test --workspace --release --locked --features tree-sitter
cargo test --workspace --release --locked --features ml
```

Run the evaluation command in `docs/evaluations.md` and inspect the changes.
Commit and push the reviewed result, wait for CI, then create and push the
matching annotated `vX.Y.Z` tag. 
The tag workflow reuses lint and test workflows for the exact tagged commit.
Version validation, default/AST/ML tests and evaluation scenarios must pass
before build jobs start. Native outputs get smoke checks. Linux ARM64 is
cross-compiled and is not runtime-smoke-tested by the x86 runner.

Artifacts include Linux x86_64/ARM64, Windows x86_64, macOS Intel/ARM64,
Linux x86_64 ML and Linux x86_64 AST variants. Every archive contains `fpe`
and the compatibility alias `first-plan-engine`. Release notes are extracted
from the matching CHANGELOG entry, and SHA256SUMS accompanies the binaries.

After publication, verify the release URL, expected seven archives and checksum
manifest. Consumers should download the corresponding archive and SHA256SUMS
and verify with `sha256sum --check --ignore-missing SHA256SUMS` (or the platform
equivalent). Checksums detect corruption; they are not signatures or provenance
attestations. Never move an already published tag to repair a release: publish
a patch version after fixing and validating the issue.

Rust builds are tested with 1.96.0. Dependencies are resolved from Cargo.lock
using `--locked`. Toolchain upgrades should be deliberate and validated.
