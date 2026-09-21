# Pilot results — 2026-09-21

Local Linux x86_64 run with Kani 0.68.0, CBMC 6.11.0 and the Kani-bundled
Rust nightly 2026-08-21. These are observed single-run results, not a performance benchmark.

| Rule | Proof result | Wall seconds | Injected defect detected |
| --- | --- | ---: | --- |
| context-budget | verified_within_scope | 0.816 | True |
| strict-contract-gate | verified_within_scope | 0.766 | True |
| instruction-preservation | verified_within_scope | 60.051 | True |

All three negative controls produced assertion failures in the expected harness.
The run with a nonexistent verifier returned `inconclusive` and exit code 1.
The collector tests cover stale inputs/logs, malformed evidence, unexpected harnesses,
missing execution, tool failure and timeout. The local report freshness check passed.

Implementation SHA-256:

`23cc3c294e3702ad60c33992575b90a52e5b05d5f4e438795cb61f1f23692621`

Harness SHA-256:

`6d19ba5509e89ab18c091c56cfdf393791520feca12865fa9f326f537051d14e`

The text property covers only 0–4 byte ASCII strings and offsets 0–5, with unwind 6.
Arithmetic/gate properties cover all 64-bit inputs. The tested production integration
and Unicode cases are regression evidence, not extensions of the formal domain.

The initial string harness reached its timeout. Validating fixed-size ASCII buffers
before slicing, preallocating the output once, and using unwind 6 reduced solver work;
the declared input domain and assertions were preserved, and unwinding checks stayed enabled.

Run `python3 tools/verify.py --mutations --output /tmp/first-plan-proof/report.json`
to obtain current evidence. The CI `verification-pilot` artifact contains the complete
report, structured Kani results, logs and specification diff for its exact checkout.
These checked-in observations are historical; they are not a substitute for rerunning
the verifier after changes. See [scope and trust assumptions](../docs/verification-pilot.md).
