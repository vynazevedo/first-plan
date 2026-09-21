#!/usr/bin/env python3
"""Conservative tripwire for proof bypasses; does not replace independent review."""
import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTECTED = ["verification", "tools/verify.py", "tools/test_verify.py",
             "tools/check_verification_policy.py", ".github/workflows/verify.yml",
             ".github/workflows/test.yml"]


def check():
    harness = (ROOT / "verification/src/lib.rs").read_text()
    production = (ROOT / "engine/crates/core/src/invariants.rs").read_text()
    # Ban common explicit escape hatches; aliases/macros and the spec still need review.
    forbidden = r"\b(assume|assume_init|assume_unreachable|admit|external_body|stub|stub_verified|should_panic|unsafe)\b"
    for name, text in [("harness", harness), ("implementation", production)]:
        code = re.sub(r"//[^\n]*", "", text)
        if re.search(forbidden, code):
            raise ValueError("proof bypass requires separate review: " + name)
    if harness.count("#[kani::proof]") != 3:
        raise ValueError("exactly three reviewed harnesses required")
    if '#[path = "../../engine/crates/core/src/invariants.rs"]' not in harness:
        raise ValueError("harness must import the actual production implementation")
    if '#[kani::unwind(6)]' not in harness:
        raise ValueError("reviewed string proof bound changed")
    print("Proof tripwires passed; specifications still require independent review.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="Trusted base commit for displaying proof-policy changes")
    args = parser.parse_args()
    check()
    if args.base:
        if not re.fullmatch(r"[0-9a-f]{40}", args.base):
            parser.error("base must be a full commit SHA")
        result = subprocess.run(["git", "diff", "--no-ext-diff", args.base, "HEAD", "--", *PROTECTED],
                                cwd=ROOT, check=True, capture_output=True, text=True)
        print("SPECIFICATION REVIEW REQUIRED" if result.stdout else "No proof-policy changes since base.")
        print(result.stdout)


if __name__ == "__main__":
    main()
