"""Valida YAML frontmatter de skills, agents e commands.

Regras aplicadas:
- Todo arquivo em `commands/`, `agents/`, `skills/` deve comecar com bloco `---`
- Campo `argument-hint` (quando presente) deve ser string, nunca list
- Campo `name` deve existir e ser string em skills e agents
- Campo `description` deve existir e ser string
- Campo `allowed-tools` (quando presente em commands) deve ser list de strings
- Bloco frontmatter deve fechar com `---` antes do body

Motivacao: bug reportado por thejesh23 (#1) - `argument-hint: [foo]` sem aspas
eh interpretado como flow-sequence YAML (array), quebrando loaders estritos
como GitHub Copilot CLI >= 1.0.65. Este validador roda no CI para pegar
regressao.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parent.parent
SCAN_DIRS = ["commands", "agents", "skills"]

STRING_FIELDS = ["name", "description", "argument-hint", "version", "color", "model"]
LIST_FIELDS = ["allowed-tools"]
# `tools` em agents pode ser string CSV ("Read, Bash") ou lista - padrao Claude Code
FLEXIBLE_LIST_FIELDS = ["tools"]


def extract_frontmatter(content: str) -> tuple[dict | None, str | None]:
    """Extrai YAML frontmatter. Retorna (data, error_msg)."""
    if not content.startswith("---"):
        return None, "arquivo nao comeca com bloco frontmatter '---'"

    lines = content.split("\n")
    end_idx = None
    for i, line in enumerate(lines[1:], start=1):
        if line.strip() == "---":
            end_idx = i
            break

    if end_idx is None:
        return None, "frontmatter nao fecha com '---'"

    yaml_body = "\n".join(lines[1:end_idx])
    try:
        data = yaml.safe_load(yaml_body)
    except yaml.YAMLError as exc:
        return None, f"YAML invalido: {exc}"

    if data is None:
        return {}, None
    if not isinstance(data, dict):
        return None, f"frontmatter nao eh um mapping (got {type(data).__name__})"

    return data, None


def validate_types(data: dict, file: Path) -> list[str]:
    errors: list[str] = []

    for field in STRING_FIELDS:
        if field not in data:
            continue
        value = data[field]
        if not isinstance(value, str):
            errors.append(
                f"campo '{field}' deve ser string, got {type(value).__name__} = {value!r}. "
                f"Provavel causa: valor sem aspas parece flow-sequence YAML (ex: '[--days N]' vira lista). "
                f"Solucao: wrap em aspas duplas."
            )

    for field in LIST_FIELDS:
        if field not in data:
            continue
        value = data[field]
        if not isinstance(value, list):
            errors.append(
                f"campo '{field}' deve ser lista, got {type(value).__name__} = {value!r}"
            )
            continue
        for i, item in enumerate(value):
            if not isinstance(item, str):
                errors.append(
                    f"campo '{field}[{i}]' deve ser string, got {type(item).__name__} = {item!r}"
                )

    for field in FLEXIBLE_LIST_FIELDS:
        if field not in data:
            continue
        value = data[field]
        if isinstance(value, str):
            continue
        if isinstance(value, list):
            for i, item in enumerate(value):
                if not isinstance(item, str):
                    errors.append(
                        f"campo '{field}[{i}]' deve ser string, got {type(item).__name__} = {item!r}"
                    )
            continue
        errors.append(
            f"campo '{field}' deve ser string CSV ou lista, got {type(value).__name__} = {value!r}"
        )

    return errors


def required_fields_for(file: Path) -> list[str]:
    if file.parent.parent.name == "skills" or file.parent.name == "skills":
        return ["name", "description"]
    if file.parent.name == "agents":
        return ["name", "description"]
    if file.parent.name == "commands":
        return ["description"]
    return []


def validate_required(data: dict, file: Path) -> list[str]:
    errors = []
    for field in required_fields_for(file):
        if field not in data:
            errors.append(f"campo obrigatorio '{field}' ausente")
        elif not data.get(field):
            errors.append(f"campo obrigatorio '{field}' vazio")
    return errors


def scan() -> int:
    errors_total = 0
    files_checked = 0

    for subdir in SCAN_DIRS:
        base = ROOT / subdir
        if not base.exists():
            continue
        for md_file in sorted(base.rglob("*.md")):
            if "meta-templates" in md_file.parts or "templates" in md_file.parts:
                continue
            if md_file.name.upper() == "README.MD":
                continue

            files_checked += 1
            content = md_file.read_text(encoding="utf-8")

            data, extract_err = extract_frontmatter(content)
            if extract_err:
                rel = md_file.relative_to(ROOT)
                print(f"FAIL {rel}: {extract_err}", file=sys.stderr)
                errors_total += 1
                continue

            if data is None:
                continue

            errs = validate_types(data, md_file)
            errs += validate_required(data, md_file)
            if errs:
                rel = md_file.relative_to(ROOT)
                for e in errs:
                    print(f"FAIL {rel}: {e}", file=sys.stderr)
                errors_total += len(errs)

    if errors_total == 0:
        print(f"OK: {files_checked} frontmatters validados sem erros")
        return 0
    print(f"\n{errors_total} erro(s) em {files_checked} arquivo(s) escaneados", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(scan())
