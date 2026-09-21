"""Validate release metadata and extract the exact changelog entry."""
import argparse
import json
import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag")
    parser.add_argument("--notes", type=Path)
    args = parser.parse_args()
    workspace = tomllib.loads((ROOT / "engine/Cargo.toml").read_text())
    version = workspace["workspace"]["package"]["version"]
    assert re.fullmatch(r"\d+\.\d+\.\d+", version), "stable SemVer required"
    if args.tag:
        assert args.tag == f"v{version}", "tag does not match workspace version"
    plugin = json.loads((ROOT / ".claude-plugin/plugin.json").read_text())
    marketplace = json.loads((ROOT / ".claude-plugin/marketplace.json").read_text())
    assert plugin["version"] == version
    assert marketplace["plugins"][0]["version"] == version
    lock = tomllib.loads((ROOT / "engine/Cargo.lock").read_text())
    for package in lock["package"]:
        if package["name"] in {"first-plan-core", "first-plan-engine"}:
            assert package["version"] == version, "Cargo.lock version mismatch"
    for path in ["README.md", "docs/i18n/README.pt-BR.md"]:
        assert f"version-{version}-" in (ROOT / path).read_text(), f"stale badge: {path}"
    changelog = (ROOT / "CHANGELOG.md").read_text()
    entry = re.search(rf"^## \[{re.escape(version)}\] - \d{{4}}-\d{{2}}-\d{{2}}\n(.*?)(?=^## \[|\Z)", changelog, re.M | re.S)
    assert entry and entry.group(1).strip(), "release needs a dated changelog entry"
    if args.notes:
        args.notes.parent.mkdir(parents=True, exist_ok=True)
        args.notes.write_text(f"# first-plan v{version}\n\n{entry.group(1).strip()}\n", encoding="utf-8")
    print(f"Release metadata consistent: v{version}")


if __name__ == "__main__":
    main()
