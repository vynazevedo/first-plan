# Project rules: context, execution and specification review

This is unreleased work after v1.5.0. It extends the engine verification pilot to
projects that use first-plan. The four pieces are a strict registry, task-specific
obligations, an explicit verifier runner, and comparison with an independently
reviewed policy. Existing v1.5.0 binaries do not contain `fpe verify`.

## 1. Register project requirements

Create `.first-plan/rules.yaml`:

```yaml
schema_version: 1
rules:
  - id: tenant-isolation
    requirement: Users may only read documents belonging to their own tenant.
    owner: application-security
    keywords: [tenant, authorization, access]
    inputs: [app]
    verification_files: [tests]
    verifier:
      kind: test
      command: [python3, -m, unittest, discover, -s, tests, -v]
      timeout_seconds: 15
```

Paths are relative files or directories, not globs. Directories are recursively
fingerprinted, including new/deleted files. Symlinks, escapes, missing inputs,
empty directories, duplicate IDs and unknown schema fields fail validation.
The registry is limited to 256KB/100 rules; snapshots to 10,000 files and a depth
of 64. List test helpers, schemas, dependency lockfiles, build scripts and other
transitive verification dependencies under `verification_files`. List mutable
implementation sources under `inputs`. The tool cannot infer every dependency.

## 2. Deliver the relevant obligations to the agent

```bash
fpe context --query "change document access" --path app/access.py --json
```

`--path` is repeatable. An explicitly declared file/directory association produces
`match_reason: declared_path`; keyword matching produces `lexical_candidate`.
MCP's existing read-only `context` tool accepts the equivalent `paths` array.
No verifier execution tool is added to MCP.

The `applicable_rules` array includes requirements, owners, verification files,
the registry SHA-256 and `evidence_status: required_not_verified`. A required rule
is not evidence that its implementation is correct. Rule obligations are separate
metadata outside the selected-content character budget, so a small budget cannot
silently omit an applicable obligation. They are bounded by the registry limits.
No dependency/call-graph completeness is claimed.

## 3. Review a policy and execute explicitly

`fpe verify` prints the registry without executing anything.

```bash
# Export a CANDIDATE. Creation is not independent approval.
fpe verify policy --out /trusted-review/candidate.json

# The responsible person reviews requirements, test coverage, command and inputs.
# An operator then supplies the approved file from trusted, read-only storage.
fpe verify review --policy /trusted-review/approved.json
fpe verify run --policy /trusted-review/approved.json --out /evidence/run-001.json
fpe verify check --policy /trusted-review/approved.json --report /evidence/run-001.json
```

`/trusted-review` and `/evidence` must exist or be creatable as appropriate; choose
real paths outside the project. Candidate export does not overwrite a file.
Execution requires an external policy with matching registry bytes, protected
verification file hashes and resolved executable hashes. `run` requires a new
output filename, preserves existing reports, runs all rules and returns nonzero
unless every rule succeeds. There is no automatic approval or policy refresh.

The CLI never interprets an implicit shell: command arguments are passed as an
argument vector. An explicitly configured shell or test runner still executes
code. Run only reviewed commands in an isolated CI environment. This runner is
not a sandbox: filesystem/network privileges, protecting the external policy and
restricting agent write access are operator responsibilities. Supplying a policy
file is the authorization boundary; the program cannot identify who reviewed it.

Commands receive no stdin and have a 1–600 second timeout. Logs are written to
files with a monitored 8MB limit (not a hard OS disk quota). Unix process groups
are terminated on completion/timeout; Windows uses `taskkill /T` plus termination
of the direct child, on a best-effort basis. OS isolation is still necessary for
untrusted commands and detached descendants. Toolchains, interpreters, ambient
environment and transitive dependencies must be controlled by CI; hashing an
executable does not identify every library or interpreter-selected toolchain.

Results distinguish:

| Status | Meaning |
| --- | --- |
| `tests_passed` | The reviewed test command exited zero; not a mathematical proof or automatic assertion-count check |
| `formally_verified` | The supported formal adapter verified the exact expected harness within the reviewed scope |
| `failed` | A test command failed or the formal checker found an assertion failure |
| `inconclusive` | Timeout, launch failure, missing/unsupported formal results, or insufficient proof evidence |
| `stale` | Inputs, reviewed policy or evidence bytes differ |

Preflight errors (missing registry/input/tool, invalid schema or policy mismatch)
abort before execution with a nonzero exit and diagnostic; they do not create a
success report. `review` reports `review_required` with added/removed rule IDs,
before/after rule definitions, registry changes, changed verifier files and executables when comparison can be
completed. Invalid or missing registries also block execution.

Reports record actual working-tree hashes, revision, command, logs, result and
elapsed time. Store the report with its adjacent `fpe-evidence-*` directory.
`check` validates current inputs, external policy, log hashes and supported formal
artifacts; it does not rerun checks or authenticate who produced them. Hashes are
not signatures, and reports are not trusted attestations on their own.

### Formal adapter: Kani 0.68.0

The first formal adapter supports Linux x86_64 and Kani's structured result format:

```yaml
verifier:
  kind: kani
  executable: cargo-kani
  working_directory: verification
  harness: proofs::budget_never_overflows_or_exceeds_limit
  scope: All 64-bit inputs of reserve_budget under the Kani Rust model.
  timeout_seconds: 180
```

It builds a fixed invocation with `--exact` and JSON export, and checks the pinned
metadata, expected harness, completed execution and assertion results. Zero exit
alone, no assertions, wrong harness, unknown checks and insufficient unwinding do
not establish a formal result. The operator must review the harness, assumptions,
bounds and solver/toolchain. The `scope` description itself is supplied by humans;
the adapter cannot establish that prose matches the theorem. Bend/Verus adapters
are not implemented.

## 4. Prevent silent specification weakening

Keep the approved policy in storage the coding agent cannot modify, ideally
read-only mounted by CI. Generate it from a separately reviewed baseline, not
from the PR being verified. Run `review` and `run` against that same approved
policy. A changed requirement, removed rule, weakened test or altered verifier
invalidates the old approval; an ordinary implementation change remains eligible
for checking. Refresh approval only after independent review.

Protect policy storage and workflow changes with your repository's review rules.
The example CI tests exercise this boundary with trusted fixtures; they do not
configure repository administrators' approval policies or certify business laws
written by the same agent. An actor who can replace both the rules and their
external approval can bypass this mechanism.

## Reproduce the tenant example

The [example](../examples/tenant-rules/.first-plan/rules.yaml) includes three real
Python tests: own-tenant access succeeds, other-tenant access fails, and missing
tenant identity fails. It is a small access-decision example, not a complete
multi-tenant application or production authorization audit.

```bash
fpe verify --root examples/tenant-rules
fpe context --root examples/tenant-rules --query authorization --path app/access.py --json
# Then follow the independent policy-review and execution sequence above,
# using --root examples/tenant-rules for each command.
```

For development/CI, the trusted end-to-end suite exports fixture policies, runs
the checks, injects a cross-tenant access defect, verifies rejection, then checks
that weakened requirements/tests cannot reuse the original policy:

```bash
python3 tools/test_project_rules.py --binary engine/target/release/fpe
# Also test the real Kani adapter when installed:
python3 tools/test_project_rules.py --binary engine/target/release/fpe --kani cargo-kani
```
