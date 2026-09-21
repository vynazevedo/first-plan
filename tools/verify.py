#!/usr/bin/env python3
"""Run the first-plan Kani pilot. Never execute commands supplied by a registry."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
RULES = ROOT / "verification/rules.json"
HARNESSES = {
    "context-budget": "budget_never_overflows_or_exceeds_limit",
    "strict-contract-gate": "strict_gate_rejects_missing_evidence",
    "instruction-preservation": "replacement_preserves_surrounding_bytes",
}
INPUTS = [
    "verification/rules.json", "verification/Cargo.toml", "verification/Cargo.lock",
    "engine/Cargo.toml", "engine/Cargo.lock", "engine/crates/core/Cargo.toml",
    "engine/crates/cli/Cargo.toml",
    "verification/src/lib.rs", "engine/crates/core/src/invariants.rs",
    "engine/crates/core/src/lib.rs", "engine/crates/core/src/context.rs",
    "engine/crates/core/src/generate/mod.rs",
    "engine/crates/cli/src/commands/contracts.rs", "engine/crates/cli/src/commands/multi.rs",
    "tools/verify.py", "tools/check_verification_policy.py",
    "tools/test_verify.py", ".github/workflows/verify.yml", ".github/workflows/test.yml",
]
MUTATIONS = {
    "context-budget": (".filter(|next| *next <= budget)", ".filter(|_next| true)"),
    "strict-contract-gate": ("!strict || (complete && breaking == 0)", "!strict || breaking == 0"),
    "instruction-preservation": ('result.push_str(&existing[..start]);', 'result.push_str("");'),
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fingerprints(root=ROOT):
    return {p: digest(root / p) for p in INPUTS}


def load_rules():
    registry = json.loads(RULES.read_text())
    if registry["schema_version"] != 1 or registry["verifier"] != {
        "name": "kani", "version": "0.68.0", "target": "x86_64-unknown-linux-gnu"
    }:
        raise ValueError("unsupported registry/verifier; review runner changes separately")
    rules = registry["rules"]
    if len(rules) != len(HARNESSES) or {r["id"]: r["harness"] for r in rules} != HARNESSES:
        raise ValueError("missing, duplicate or unexpected verification rules")
    return registry


def invoke(argv, cwd, timeout, log):
    start = time.monotonic()
    try:
        with log.open("w") as out:
            child = subprocess.Popen(argv, cwd=cwd, stdout=out, stderr=subprocess.STDOUT,
                                     start_new_session=True)
            try:
                code = child.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                # Kill the verifier and its solver/compiler children, not only cargo.
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                return None, time.monotonic() - start, "timeout"
        return code, time.monotonic() - start, None
    except OSError as error:
        log.write_text(str(error))
        return None, time.monotonic() - start, str(error)


def classify(code, artifact, harness):
    """Accept exactly one completed pinned-verifier result, with actual assertions."""
    if code is None:
        return "inconclusive"
    try:
        data = json.loads(artifact.read_text())
        metadata = data["metadata"]
        if metadata["version"] != "1.0" or metadata["kani_version"] != "0.68.0" or metadata["target"] != "x86_64-unknown-linux-gnu":
            return "inconclusive"
        verification = data["verification_results"]
        summary = verification["summary"]
        results = verification["results"]
        if summary["status"] != "completed" or summary["executed"] != 1 or summary["total_harnesses"] != 1 or len(results) != 1:
            return "inconclusive"
        result = results[0]
        if result["harness_id"] != "proofs::" + harness:
            return "inconclusive"
        checks = result["checks"]
        if not any(c["category"] == "assertion" for c in checks):
            return "inconclusive"
        # An unwinding failure, solver error or unknown result is never a counterexample.
        if any(c["status"] not in {"Success", "Failure", "Unreachable"} for c in checks):
            return "inconclusive"
        failed = [c for c in checks if c["status"] == "Failure"]
        if any(c["category"] != "assertion" for c in failed):
            return "inconclusive"
        if code == 0 and result["status"] == "Success" and not failed and summary["successful"] == 1 and summary["failed"] == 0:
            return "verified_within_scope"
        if code != 0 and result["status"] == "Failure" and failed and summary["failed"] == 1:
            # Require the failing assertion to originate in the reviewed harness.
            if any(c.get("location", {}).get("file") == "src/lib.rs" for c in failed):
                return "failed"
        return "inconclusive"
    except (OSError, ValueError, KeyError, TypeError):
        return "inconclusive"


def command_for(tool, harness, artifact):
    return [tool, "kani", "--harness", "proofs::" + harness, "--exact",
            "--output-format", "regular", "-Z", "unstable-options", "--export-json", str(artifact)]


def check_report(path, root=ROOT):
    report = json.loads(path.read_text())
    if report.get("schema_version") != 1 or report.get("inputs") != fingerprints(root):
        return "stale"
    if report.get("status") != "verified_within_scope":
        return "inconclusive"
    results = report.get("results", [])
    if len(results) != 3 or {r.get("id") for r in results} != set(HARNESSES):
        return "inconclusive"
    if any(r.get("status") != "verified_within_scope" for r in results):
        return "inconclusive"
    for result in results:
        for field in ("log", "artifact"):
            artifact = path.parent / result.get(field, "")
            if not artifact.is_file() or digest(artifact) != result.get(field + "_sha256"):
                return "stale"
        if classify(result.get("exit_code"), path.parent / result["artifact"], HARNESSES[result["id"]]) != "verified_within_scope":
            return "inconclusive"
    if report.get("mutations_requested"):
        controls = report.get("negative_controls", [])
        if len(controls) != 3 or {c.get("id") for c in controls} != set(HARNESSES):
            return "inconclusive"
        for control in controls:
            for field in ("log", "artifact"):
                artifact = path.parent / control.get(field, "")
                if not artifact.is_file() or digest(artifact) != control.get(field + "_sha256"):
                    return "stale"
            if not control.get("detected") or classify(control.get("exit_code"), path.parent / control["artifact"], HARNESSES[control["id"]]) != "failed":
                return "inconclusive"
    # This validates a local report's freshness, not cryptographic authenticity.
    return "verified_within_scope"


def run(args):
    registry = load_rules()
    forbidden = [k for k in os.environ if k.startswith(("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "KANIFLAGS", "CBMC_FLAGS"))]
    if forbidden:
        raise ValueError("verification flag overrides are not accepted: " + ", ".join(forbidden))
    output = args.output.resolve()
    if output in {(ROOT / p).resolve() for p in INPUTS}:
        raise ValueError("report must not overwrite verification inputs")
    output.parent.mkdir(parents=True, exist_ok=True)
    logs = output.parent / (output.stem + "-logs")
    logs.mkdir(exist_ok=True)
    before = fingerprints()
    report = {
        "schema_version": 1,
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "inputs": before, "verifier": registry["verifier"], "rules": registry["rules"],
        "trust": registry["trust"], "results": [], "negative_controls": [],
        "mutations_requested": args.mutations,
    }
    rev = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True)
    report["revision"] = rev.stdout.strip() if rev.returncode == 0 else None
    tool = str(Path(args.kani).resolve()) if os.path.sep in args.kani else args.kani
    version_log = logs / "version.log"
    code, _, error = invoke([tool, "kani", "--version"], ROOT, 30, version_log)
    version = version_log.read_text(errors="replace").strip()
    report["verifier_output"] = version
    ready = code == 0 and version == "Kani Rust Verifier 0.68.0 (cargo plugin)\nCBMC 6.11.0" and platform.system() == "Linux" and platform.machine() == "x86_64"
    for rule in registry["rules"]:
        log = logs / (rule["id"] + ".log")
        artifact = log.with_suffix(".json")
        artifact.unlink(missing_ok=True)
        command = command_for(tool, rule["harness"], artifact)
        if ready:
            code, elapsed, error = invoke(command, ROOT / "verification", args.timeout, log)
            status = classify(code, artifact, rule["harness"])
        else:
            code, elapsed, status = None, 0, "inconclusive"
            error = "Kani 0.68.0 on Linux x86_64 required; " + (error or version)
            log.write_text(error)
        report["results"].append({"id": rule["id"], "status": status, "command": command,
            "exit_code": code, "elapsed_seconds": round(elapsed, 3), "error": error,
            "log": str(log.relative_to(output.parent)), "log_sha256": digest(log),
            "artifact": str(artifact.relative_to(output.parent)), "artifact_sha256": digest(artifact) if artifact.exists() else None})
    if args.mutations and ready and all(r["status"] == "verified_within_scope" for r in report["results"]):
        for rule in registry["rules"]:
            # Copy sources into a temporary workspace; production is never mutated.
            with tempfile.TemporaryDirectory(prefix="first-plan-negative-control-") as directory:
                tmp = Path(directory)
                for name in ["verification/Cargo.toml", "verification/Cargo.lock", "verification/src/lib.rs", "engine/crates/core/src/invariants.rs"]:
                    dest = tmp / name
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    dest.write_bytes((ROOT / name).read_bytes())
                source = tmp / "engine/crates/core/src/invariants.rs"
                old, new = MUTATIONS[rule["id"]]
                text = source.read_text()
                if text.count(old) != 1:
                    raise ValueError("mutation target drifted: " + rule["id"])
                source.write_text(text.replace(old, new))
                log = logs / (rule["id"] + "-negative.log")
                artifact = log.with_suffix(".json")
                artifact.unlink(missing_ok=True)
                command = command_for(tool, rule["harness"], artifact)
                code, elapsed, error = invoke(command, tmp / "verification", args.timeout, log)
                status = classify(code, artifact, rule["harness"])
                report["negative_controls"].append({"id": rule["id"], "detected": status == "failed",
                    "status": status, "exit_code": code, "elapsed_seconds": round(elapsed, 3),
                    "error": error, "log": str(log.relative_to(output.parent)), "log_sha256": digest(log),
                    "artifact": str(artifact.relative_to(output.parent)), "artifact_sha256": digest(artifact) if artifact.exists() else None,
                    "mutation": {"before": old, "after": new}})
    report["status"] = "verified_within_scope" if all(r["status"] == "verified_within_scope" for r in report["results"]) else "inconclusive"
    if args.mutations and (len(report["negative_controls"]) != 3 or not all(r["detected"] for r in report["negative_controls"])):
        report["status"] = "inconclusive"
    if any(r["status"] == "failed" for r in report["results"]):
        report["status"] = "failed"
    if fingerprints() != before:
        report["status"] = "stale"
    output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "report": str(output)}))
    return 0 if report["status"] == "verified_within_scope" else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kani", default="cargo-kani", help="Explicit trusted cargo-kani executable")
    parser.add_argument("--output", type=Path, default=Path("verification-report.json"))
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--mutations", action="store_true")
    parser.add_argument("--check-report", type=Path, help="Check local input freshness; does not rerun proofs or authenticate results")
    args = parser.parse_args()
    if not 1 <= args.timeout <= 600:
        parser.error("timeout must be 1..600 seconds per harness")
    if args.check_report:
        status = check_report(args.check_report)
        print(json.dumps({"status": status, "assurance": "local_report_freshness_only"}))
        return 0 if status == "verified_within_scope" else 1
    return run(args)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(json.dumps({"status": "inconclusive", "error": str(error)}))
        sys.exit(1)
