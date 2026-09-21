"""Regression tests for the evidence collector, independent of a Kani install."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("verify", Path(__file__).with_name("verify.py"))
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


def result_fixture(harness, status="Success"):
    failed = status == "Failure"
    return {
        "metadata": {"version": "1.0", "kani_version": "0.68.0", "target": "x86_64-unknown-linux-gnu"},
        "verification_results": {
            "summary": {"status": "completed", "executed": 1, "total_harnesses": 1,
                        "successful": int(not failed), "failed": int(failed)},
            "results": [{"harness_id": "proofs::" + harness, "status": status,
                         "checks": [{"category": "assertion", "status": status,
                                     "location": {"file": "src/lib.rs"}}]}],
        },
    }


class CollectorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.artifact = self.root / "result.json"
        self.harness = verify.HARNESSES["context-budget"]

    def classify(self, data, code=0):
        self.artifact.write_text(json.dumps(data))
        return verify.classify(code, self.artifact, self.harness)

    def test_requires_expected_harness_and_completed_execution(self):
        data = result_fixture(self.harness)
        self.assertEqual(self.classify(data), "verified_within_scope")
        for edit in (
            lambda d: d["verification_results"]["summary"].update(executed=0),
            lambda d: d["verification_results"]["summary"].update(status="timeout"),
            lambda d: d["verification_results"]["results"][0].update(harness_id="different"),
            lambda d: d["metadata"].update(kani_version="other"),
            lambda d: d["verification_results"]["results"][0].update(checks=[]),
        ):
            changed = copy.deepcopy(data)
            edit(changed)
            self.assertEqual(self.classify(changed), "inconclusive")
        self.assertEqual(self.classify(data, code=None), "inconclusive")
        self.assertEqual(self.classify(data, code=1), "inconclusive")

    def test_only_harness_assertions_count_as_detected_mutations(self):
        data = result_fixture(self.harness, "Failure")
        self.assertEqual(self.classify(data, 1), "failed")
        check = data["verification_results"]["results"][0]["checks"][0]
        for category in ["unwind", "pointer_dereference", "overflow"]:
            check["category"] = category
            self.assertEqual(self.classify(data, 1), "inconclusive")
        check["category"] = "assertion"
        check["location"]["file"] = "some_dependency.rs"
        self.assertEqual(self.classify(data, 1), "inconclusive")

    def test_exit_zero_with_missing_or_malformed_artifact_is_not_a_proof(self):
        self.assertEqual(verify.classify(0, self.artifact, self.harness), "inconclusive")
        self.artifact.write_text("VERIFICATION:- SUCCESSFUL")
        self.assertEqual(verify.classify(0, self.artifact, self.harness), "inconclusive")

    def test_report_rejects_changed_sources_logs_and_results(self):
        for name in verify.INPUTS:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture")
        report = {"schema_version": 1, "inputs": verify.fingerprints(self.root),
                  "status": "verified_within_scope", "results": []}
        for rule_id, harness in verify.HARNESSES.items():
            log = self.root / (rule_id + ".log")
            log.write_text("fixture log")
            artifact = log.with_suffix(".json")
            artifact.write_text(json.dumps(result_fixture(harness)))
            report["results"].append({"id": rule_id, "status": "verified_within_scope",
                "exit_code": 0, "log": str(log), "log_sha256": verify.digest(log),
                "artifact": str(artifact), "artifact_sha256": verify.digest(artifact)})
        path = self.root / "report.json"
        path.write_text(json.dumps(report))
        self.assertEqual(verify.check_report(path, self.root), "verified_within_scope")
        log.write_text("changed")
        self.assertEqual(verify.check_report(path, self.root), "stale")
        log.write_text("fixture log")
        (self.root / verify.INPUTS[0]).write_text("changed rules")
        self.assertEqual(verify.check_report(path, self.root), "stale")

    def test_timeout_and_missing_executable_are_inconclusive(self):
        log = self.root / "process.log"
        code, _, error = verify.invoke(["/does/not/exist"], self.root, 1, log)
        self.assertIsNone(code)
        self.assertTrue(error)
        import sys
        code, _, error = verify.invoke([sys.executable, "-c", "import time; time.sleep(10)"], self.root, 1, log)
        self.assertIsNone(code)
        self.assertEqual(error, "timeout")


if __name__ == "__main__":
    unittest.main()
