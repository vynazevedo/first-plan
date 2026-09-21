# Evaluate outcomes before expanding scope

Run the deterministic suite:

```bash
cargo build --manifest-path engine/Cargo.toml --release --locked --bin fpe
python3 tools/evaluate.py --binary engine/target/release/fpe --output /tmp/evaluation.json
```

It checks three synthetic maintenance scenarios: reuse and test evidence for
email validation, preservation of existing team instructions across repeated
generation, and rejection of a newly required API parameter. CI stores the JSON
report. These scenarios are regression evidence, not proof of better agent
performance or production effectiveness.

For a real paired study, use the same task, repository commit, model, tool
permissions, time limit and trial seed in two isolated worktrees. The baseline
gets ordinary repository access. The treatment additionally gets first-plan
context. Let the same task-specific tests judge correctness; manually review
duplicates and false positives using a rubric established before running.
Alternate arm order and repeat tasks to account for variance. Preserve prompts,
model/provider versions, transcripts and test logs outside generated context.

Choose tasks with realistic failure modes:

1. Add validation where an existing helper and its tests should be reused.
2. Change an API schema with a downstream client that must remain compatible.
3. Fix a regression where released and deployed commits differ.

Export measured outcomes as a JSON array. Each trial needs both arms:

```json
[
  {"task":"TASK_ID","model":"MODEL_VERSION","trial":1,"arm":"baseline","success":false,"tokens":1000,"elapsed_ms":20000,"duplicates":1,"false_positives":0},
  {"task":"TASK_ID","model":"MODEL_VERSION","trial":1,"arm":"first_plan","success":true,"tokens":900,"elapsed_ms":18000,"duplicates":0,"false_positives":0}
]
```

Those numbers illustrate the format only; they are not measured results.

```bash
python3 tools/evaluate.py --binary engine/target/release/fpe \
  --paired-results measured-trials.json --output /tmp/paired-report.json
```

The aggregator rejects missing pairs and duplicate arms and reports mean
success, tokens, elapsed time, duplicate implementations and false positives.
It does not infer statistical significance or causal gains from a small sample.
The v1.5 release does not claim completed real-agent paired trials.
