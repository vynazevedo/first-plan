# Verification pilot execution plan

Objective: establish three narrowly scoped properties of the Rust implementation
that first-plan actually executes, with independently checked evidence and
negative controls. This is not a claim of whole-program correctness.

1. Extract dependency-free production primitives for budget reservation, strict
   contract gate decisions and replacement of a text range. Call them from the
   existing CLI/core paths. Preserve existing behavior except rejecting overflow.
2. Import that exact source file into a separate Kani harness workspace. Pin
   Kani 0.68.0; state the input domains and assumptions beside each property.
3. Define a reviewable rule registry with owners, scope, verifier and limits.
   Produce SHA-256-bound reports with tool identity, invocation, logs and explicit
   passed/failed/inconclusive/stale states. Missing tools never produce success.
4. Run three negative controls against temporary copies: erase an instruction
   prefix, accept incomplete analysis, and ignore a budget limit. A control counts
   as detected only for a completed verification with a property failure, not for
   a compiler failure or timeout.
5. Add a reusable CI job (also included in release gates), store the reports and
   logs, and expose changes to the verification specification for independent
   review. No automatic rule weakening, proof assumptions or solver bypass flags.
6. Run regression, integration, lint, verification-runner and mutation checks.
   Document observed results and limits before publication.

Acceptance: all three properties pass in the pinned environment, all three
negative controls fail for the intended assertion, stale reports are rejected,
missing tools/timeouts cannot pass, and existing runtime regression tests pass.

Out of scope: a generic command runner for arbitrary repositories, a universal
`fpe verify`, Bend/Verus integrations, live MCP execution, automatic business-law
inference, formal verification of filesystem behavior/parsers, and claims of
measured real-agent productivity. These require separate evaluation after this
pilot; no new stable release is implied by the experiment.
