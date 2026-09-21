#!/usr/bin/env python3
"""End-to-end project rule checks using the compiled CLI and a real tenant fixture."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[1]
BINARY = None
KANI = None


class ProjectRulesTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="fpe-project-rules-")
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name)
        self.root = self.home / "app"
        shutil.copytree(REPO / "examples/tenant-rules", self.root, ignore=shutil.ignore_patterns("__pycache__"))
        registry = self.root / ".first-plan/rules.yaml"
        registry.write_text(registry.read_text().replace("command: [python3,", "command: [" + json.dumps(os.sys.executable) + ","))
        self.policy = self.home / "reviewed.json"

    def cli(self, *args, ok=True):
        result = subprocess.run([str(BINARY), "verify", "--root", str(self.root), *map(str, args)], capture_output=True, text=True, timeout=90)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def authorize_fixture(self):
        # This trusted test harness supplies the policy; production requires independent review.
        candidate = json.loads(self.cli("policy", "--out", self.policy).stdout)
        self.assertEqual(candidate["status"], "review_required")

    def test_context_plan_pass_failure_and_staleness(self):
        self.assertEqual(len(json.loads(self.cli().stdout)["rules"]), 1)
        context = subprocess.run([str(BINARY), "context", "--root", str(self.root), "--query", "unrelated", "--path", "app/access.py", "--budget", "256", "--json"], capture_output=True, text=True, check=True)
        obligation = json.loads(context.stdout)["applicable_rules"][0]
        self.assertEqual(obligation["id"], "tenant-isolation")
        self.assertEqual(obligation["match_reason"], "declared_path")
        self.authorize_fixture()
        good = self.home / "good.json"
        self.cli("run", "--policy", self.policy, "--out", good)
        report = json.loads(good.read_text())
        self.assertEqual(report["results"][0]["status"], "tests_passed")
        self.cli("check", "--policy", self.policy, "--report", good)
        source = self.root / "app/access.py"
        original = source.read_text()
        source.write_text(original.replace('actor_tenant != document["tenant_id"]', 'False'))
        self.cli("review", "--policy", self.policy)  # ordinary code changes remain authorized
        bad = self.home / "bad.json"
        self.cli("run", "--policy", self.policy, "--out", bad, ok=False)
        self.assertEqual(json.loads(bad.read_text())["status"], "failed")
        self.cli("check", "--policy", self.policy, "--report", good, ok=False)
        source.write_text(original)
        self.cli("check", "--policy", self.policy, "--report", good)
        log = self.home / report["results"][0]["log"]
        log.write_text("forged")
        self.cli("check", "--policy", self.policy, "--report", good, ok=False)

    def test_weakened_requirements_and_tests_cannot_reuse_policy(self):
        self.authorize_fixture()
        registry = self.root / ".first-plan/rules.yaml"
        original = registry.read_text()
        registry.write_text(original.replace("Missing tenant identity must be rejected.", "Missing identity is allowed."))
        review = self.cli("review", "--policy", self.policy, ok=False)
        self.assertEqual(json.loads(review.stdout)["status"], "review_required")
        out = self.home / "unauthorized.json"
        self.cli("run", "--policy", self.policy, "--out", out, ok=False)
        self.assertFalse(out.exists())
        registry.write_text(original)
        test = self.root / "tests/test_access.py"
        test.write_text("# all tests deleted\n")
        self.cli("run", "--policy", self.policy, "--out", out, ok=False)
        self.assertFalse(out.exists())

    def test_missing_registry_verifier_and_internal_policy_cannot_pass(self):
        self.authorize_fixture()
        inside = self.root / "approval.json"
        shutil.copyfile(self.policy, inside)
        self.cli("run", "--policy", inside, "--out", self.home / "no.json", ok=False)
        registry = self.root / ".first-plan/rules.yaml"
        registry.write_text(registry.read_text().replace(json.dumps(os.sys.executable), '"missing-fpe-verifier-000"'))
        self.cli("run", "--policy", self.policy, "--out", self.home / "no.json", ok=False)
        registry.unlink()
        self.cli(ok=False)

    def test_pinned_kani_adapter_against_real_verifier(self):
        if not KANI:
            self.skipTest("provide --kani to exercise the installed formal verifier")
        for name in ["verification/src/lib.rs", "verification/Cargo.toml", "verification/Cargo.lock", "engine/crates/core/src/invariants.rs"]:
            dest = self.root / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / name, dest)
        registry = {"schema_version": 1, "rules": [{"id": "budget", "requirement": "Reservation fits the character budget without overflow", "owner": "maintainers", "inputs": ["engine/crates/core/src/invariants.rs"], "verification_files": ["verification/src/lib.rs", "verification/Cargo.toml", "verification/Cargo.lock"], "verifier": {"kind": "kani", "executable": str(KANI), "working_directory": "verification", "harness": "proofs::budget_never_overflows_or_exceeds_limit", "scope": "All 64-bit inputs; source-level Kani model", "timeout_seconds": 60}}]}
        (self.root / ".first-plan/rules.yaml").write_text(json.dumps(registry))
        self.authorize_fixture()
        out = self.home / "formal.json"
        self.cli("run", "--policy", self.policy, "--out", out)
        self.assertEqual(json.loads(out.read_text())["results"][0]["status"], "formally_verified")
        self.cli("check", "--policy", self.policy, "--report", out)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--kani")
    args = parser.parse_args()
    BINARY = args.binary.resolve()
    if args.kani:
        KANI = Path(shutil.which(args.kani) or args.kani).resolve()
    unittest.main(argv=[__file__])
