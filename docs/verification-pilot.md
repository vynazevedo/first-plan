# Verification pilot: rules checked against production Rust

This is development work after v1.5.0, not a feature shipped in that release.
The pilot adopts the separation of requirements and independently checked
implementation evidence illustrated by Bend's LAWS/PROOF workflow. It keeps the
engine in Rust and uses Kani 0.68.0 on Linux x86_64. No LLM is invoked.

The [execution plan](../verification/PLAN.md), [observed results](../verification/RESULTS.md) and [rule registry](../verification/rules.json)
define the scope. The harness imports the **same source file** used by the engine,
not a translated model or a duplicate implementation.

| Rule | Production usage | Formal scope |
| --- | --- | --- |
| Context budget | `fpe context` reserves each selected item's cost | All 64-bit unsigned inputs; mathematical sum must fit the budget, including overflow rejection |
| Strict contract gate | Local and cross-repository contract diff gates | All boolean inputs and 64-bit breaking counts; missing evidence cannot pass strict mode |
| Instruction preservation | `fpe generate` replaces an existing managed block | All ASCII source/replacement strings of 0–4 bytes; byte offsets 0–5; unwind limit 6 |

The string bound is intentionally small and explicit. Unicode/CRLF, full marker
handling and the connection to CLI behavior have regression tests; those tests
are not universal formal proofs. The gate proof does not establish that the
OpenAPI analyzer found every compatibility problem. Budget verification does not
cover candidate ranking, cost computation or the JSON envelope. Filesystem
atomicity, symlink races, parsing and whole-application correctness are not proved.

## Reproduce

Kani downloads its own compiler toolchain. The engine continues to use Rust
1.96.0; the dependency-free verification workspace is separate. These are source-level
checks through Kani's Rust frontend, not equivalence certificates for the release
compiler or native binaries. Rust/standard-library modeling and code generation
remain part of the trust boundary.

```bash
cargo install --locked kani-verifier --version 0.68.0
cargo kani setup
python3 tools/check_verification_policy.py
python3 -m unittest discover -s tools -p test_verify.py
python3 tools/verify.py --mutations --output /tmp/first-plan-proof/report.json
python3 tools/verify.py --check-report /tmp/first-plan-proof/report.json
```

`--kani /absolute/path/to/cargo-kani` selects an explicitly trusted installation;
`KANI_HOME` can relocate Kani's bundle. The registry cannot supply executable
commands. Run verification only in a trusted checkout or isolated CI environment:
Cargo and the compiler process repository code. This is not an arbitrary-project
command runner and does not expose code execution through MCP.

Each harness has a default 180-second wall-clock timeout (`--timeout`, maximum
600). A timeout kills the process group, including solver/compiler children.
Missing tools, wrong versions, unfinished checks, unexpected output, solver
errors and insufficient loop unwinding cannot pass. Standard verification checks
remain enabled; no assumptions or stubs are added to the harnesses. Kani's own
standard-library models belong to the trusted verification toolchain.

## Evidence and states

The report records source, harness, registry, caller, build-metadata and runner
SHA-256 hashes; Git revision; exact command; elapsed time; verifier identity;
logs; and Kani's structured JSON, including compiler/solver metadata.

- `verified_within_scope`: the expected harness completed successfully with checks.
- `failed`: the reviewed harness produced an assertion counterexample.
- `inconclusive`: missing evidence, timeout, tool failure or an unsupported outcome.
- `stale`: a recorded input or evidence artifact differs from the current bytes.

`--check-report` checks local consistency/freshness and returns nonzero for stale
or incomplete evidence. It does not execute the proof again. A report is not a
signed attestation: someone controlling the report and logs can forge them.
Trust CI execution and review the assumptions; SHA-256 alone does not establish
authenticity. Archive the report together with its adjacent `*-logs/` directory.

## Negative controls

`--mutations` creates independent temporary workspaces, introduces one deliberate
bug per rule and reruns the corresponding unmodified harness:

1. Ignore the budget limit.
2. Accept incomplete analysis in strict mode.
3. Delete the prefix outside a replaced text range.

The engine checkout is never modified by these controls. A mutation counts as
detected only when a completed Kani run finds a harness assertion failure.
A compilation failure or timeout is inconclusive, not a successful detection.
These controls show sensitivity to three specific bugs, not a general defect
coverage percentage or measured improvement in AI performance.

## Review and CI

The reusable verification job runs from the test workflow, which also gates
future releases. It runs the three proofs, three negative controls, collector
tests and freshness checks, and stores evidence as the `verification-pilot`
artifact. This adds a development/release gate without changing v1.5.0 artifacts.

The policy checker rejects common explicit bypasses and emits a diff of proof
policy files against the trusted base commit. It is a tripwire, not a complete
Rust parser or a security boundary. A green check does not approve a weakened
specification: maintainers must independently review changes to requirements,
harnesses, bounds, the verifier and its runner. Repository rules requiring such
review are an administrative follow-up, not something this pilot claims to enforce.

## Next decision

Use measured proof runtime, counterexamples and maintenance effort to decide
whether to expand bounds or try Verus for unbounded string reasoning. The [project rules workflow](project-rules.md) now adds `fpe verify` with external
policies, a test adapter and a pinned Kani adapter. Bend adapters and inferred
business rules remain future work. The original three-property pilot remains
reproducible through `tools/verify.py`.

Primary references: [Bend laws and proofs](https://github.com/bendlang/bend/blob/main/guide/GUIDE.md),
[Kani installation](https://model-checking.github.io/kani/install-guide.html),
[Kani loop bounds](https://model-checking.github.io/kani/reference/attributes.html),
[Kani 0.68.0 structured results](https://github.com/model-checking/kani/releases/tag/kani-0.68.0),
and [Verus guidance on protecting specifications from LLM shortcuts](https://verus-lang.github.io/verus/guide/llmforverusproof.html).
