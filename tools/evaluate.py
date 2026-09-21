"""Reproducible retrieval/contract checks, plus optional real paired agent outcomes.

No model calls, credentials or estimated success gains. Paired outcomes must be
measured externally with identical task, model and trial in each arm.
"""
import argparse
import json
import subprocess
import tempfile
import time
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--paired-results", type=Path)
    args = parser.parse_args()
    binary = str(args.binary.resolve())
    results = []
    with tempfile.TemporaryDirectory(prefix="first-plan-eval-") as directory:
        root = Path(directory)
        (root / "validation.py").write_text("def validate_email(value):\n    return '@' in value\n")
        (root / "test_validation.py").write_text("def test_validate_email():\n    assert validate_email('a@b')\n")
        (root / "unrelated.py").write_text("def format_currency(cents): return str(cents)\n")
        (root / "CONVENTIONS.md").write_text("Reuse validate_email before adding an email validator.\n")

        def run(*command):
            return subprocess.run([binary, *command, "--root", str(root)], capture_output=True, text=True, check=True)

        start = time.perf_counter()
        pack = json.loads(run("context", "--query", "validate email", "--budget", "3000", "--json").stdout)
        paths = {i["evidence"]["path"] for i in pack["items"]}
        assert {"validation.py", "test_validation.py", "CONVENTIONS.md"} <= paths
        assert "unrelated.py" not in paths
        results.append({"task": "reuse_with_validation_evidence", "passed": True,
                        "elapsed_ms": round((time.perf_counter() - start) * 1000, 2),
                        "payload_chars": len(json.dumps(pack)), "selected_chars": pack["content_chars"]})

        initial = "# Team instructions\nKeep this rule.\n"
        (root / "AGENTS.md").write_text(initial)
        run("generate", "--tool", "codex")
        first = (root / "AGENTS.md").read_text()
        run("generate", "--tool", "codex")
        assert (root / "AGENTS.md").read_text() == first and first.startswith(initial.rstrip())
        results.append({"task": "preserve_team_instructions", "passed": True})

        spec = {"openapi": "3.0.3", "info": {"title": "fixture", "version": "1"},
                "paths": {"/users": {"get": {"operationId": "listUsers", "parameters": [], "responses": {"200": {"description": "OK"}}}}}}
        (root / "openapi.json").write_text(json.dumps(spec))
        baseline = root / "baseline.json"
        run("contracts", "snapshot", "--out", str(baseline))
        spec["paths"]["/users"]["get"]["parameters"] = [{"in": "query", "name": "tenant", "required": True, "schema": {"type": "string"}}]
        (root / "openapi.json").write_text(json.dumps(spec))
        diff = json.loads(run("contracts", "diff", "--before", str(baseline), "--json").stdout)
        assert diff["summary"]["breaking"] > 0 and not diff["warnings"]
        results.append({"task": "detect_required_parameter_regression", "passed": True})

    report = {"schema_version": 1, "engine": subprocess.check_output([binary, "--version"], text=True).strip(),
              "scope": "deterministic synthetic regression scenarios; not an agent effectiveness study",
              "scenarios": results, "agent_ab": None}
    if args.paired_results:
        records = json.loads(args.paired_results.read_text())
        assert records, "paired results must not be empty"
        pairs = {}
        for row in records:
            key = (row["task"], row["model"], row["trial"])
            assert row["arm"] in {"baseline", "first_plan"}
            assert isinstance(row["success"], bool)
            for metric in ["tokens", "elapsed_ms", "duplicates", "false_positives"]:
                assert isinstance(row[metric], (int, float)) and row[metric] >= 0
            pair = pairs.setdefault(key, {})
            assert row["arm"] not in pair, "duplicate trial arm"
            pair[row["arm"]] = row
        assert all(set(p) == {"baseline", "first_plan"} for p in pairs.values()), "unpaired trials"
        report["agent_ab"] = {"paired_trials": len(pairs), "arms": {}}
        for arm in ["baseline", "first_plan"]:
            rows = [pair[arm] for pair in pairs.values()]
            report["agent_ab"]["arms"][arm] = {metric: sum(r[metric] for r in rows) / len(rows)
                                              for metric in ["success", "tokens", "elapsed_ms", "duplicates", "false_positives"]}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(f"{len(results)} scenarios passed; report: {args.output}")


if __name__ == "__main__":
    main()
