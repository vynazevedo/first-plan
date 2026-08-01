//! Detection of deprecated code items via comments and CHANGELOG.
//!
//! Fontes:
//! 1. Comentarios in-code: @deprecated, DEPRECATED:, TODO(deprecate), TODO(remove-after: DATE)
//! 2. CHANGELOG.md secoes "Deprecated" ou "Removed" (Keep-a-Changelog format)
//! 3. Rust: #[deprecated], #[deprecated(since, note)]
//! 4. Python: @deprecated, warnings.warn(DeprecationWarning)
//! 5. Java/Kotlin: @Deprecated
//! 6. JS/TS: @deprecated JSDoc
//!
//! Output: cada item com replacement (quando conhecido) e severity.

use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeprecationsReport {
    pub items: Vec<DeprecatedItem>,
    pub changelog_deprecations: Vec<ChangelogEntry>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedItem {
    pub file: String,
    pub line: u32,
    pub marker: String,
    pub context: String,
    pub replacement: Option<String>,
    pub since: Option<String>,
    pub remove_after: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub kind: ChangelogKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChangelogKind {
    Deprecated,
    Removed,
    Changed,
}

const EXCLUDED_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "vendor",
    ".git",
    "dist",
    "build",
    ".first-plan",
];

const SOURCE_EXTS: &[&str] = &[
    "rs", "go", "py", "ts", "tsx", "js", "jsx", "java", "kt", "rb", "php", "cs", "swift", "scala",
];

const MAX_FILES: usize = 5000;

pub fn detect(root: &Path) -> DeprecationsReport {
    let mut report = DeprecationsReport::default();

    scan_source_files(root, &mut report);
    parse_changelog(root, &mut report);

    report.total = report.items.len() + report.changelog_deprecations.len();
    report
}

fn scan_source_files(root: &Path, report: &mut DeprecationsReport) {
    let mut scanned = 0;
    for entry in WalkDir::new(root)
        .max_depth(10)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|d| name == *d)
        })
        .filter_map(|e| e.ok())
    {
        if scanned >= MAX_FILES {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !SOURCE_EXTS.contains(&ext) {
            continue;
        }
        scanned += 1;
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) if c.len() < 500_000 => c,
            _ => continue,
        };
        let rel = entry
            .path()
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| entry.path().to_string_lossy().into_owned());

        scan_content(&rel, &content, report);
    }
}

fn scan_content(file: &str, content: &str, report: &mut DeprecationsReport) {
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(item) = detect_deprecation_marker(file, idx, trimmed, line) {
            report.items.push(item);
        }
    }
}

fn detect_deprecation_marker(
    file: &str,
    idx: usize,
    trimmed: &str,
    original: &str,
) -> Option<DeprecatedItem> {
    let lower = trimmed.to_lowercase();

    let marker = if lower.contains("@deprecated")
        || trimmed.contains("#[deprecated")
        || trimmed.contains("@Deprecated")
    {
        "annotation".to_string()
    } else if lower.contains("todo(deprecate")
        || lower.contains("todo(remove")
        || lower.contains("todo: deprecate")
        || lower.contains("todo: remove")
    {
        "todo".to_string()
    } else if lower.starts_with("// deprecated")
        || lower.starts_with("# deprecated")
        || lower.starts_with("/* deprecated")
        || lower.starts_with("deprecated:")
        || trimmed.contains("DEPRECATED:")
    {
        "comment".to_string()
    } else if lower.contains("deprecationwarning") {
        "runtime-warn".to_string()
    } else {
        return None;
    };

    let replacement = extract_replacement(trimmed);
    let since = extract_since(trimmed);
    let remove_after = extract_remove_after(trimmed);

    let severity = if remove_after.is_some() {
        Severity::Critical
    } else if since.is_some() || replacement.is_some() {
        Severity::Warning
    } else {
        Severity::Info
    };

    Some(DeprecatedItem {
        file: file.to_string(),
        line: (idx + 1) as u32,
        marker,
        context: original.trim().to_string(),
        replacement,
        since,
        remove_after,
        severity,
    })
}

fn extract_replacement(line: &str) -> Option<String> {
    for token in [
        "use ",
        "replaced by ",
        "replaced-by:",
        "use instead:",
        "prefer ",
    ] {
        if let Some(idx) = line.to_lowercase().find(token) {
            let after = &line[idx + token.len()..];
            let end = after
                .find(['\n', ')', '"', '\''])
                .unwrap_or(after.len().min(80));
            let candidate = after[..end].trim().trim_end_matches(&['.', ',', ';'][..]);
            if !candidate.is_empty() && candidate.len() < 100 {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

fn extract_since(line: &str) -> Option<String> {
    let patterns = ["since = \"", "since=\"", "since: ", "@since "];
    for p in patterns {
        if let Some(idx) = line.find(p) {
            let after = &line[idx + p.len()..];
            let end = after
                .find(['"', ')', ',', ' ', '\n'])
                .unwrap_or(after.len().min(30));
            let candidate = after[..end].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

fn extract_remove_after(line: &str) -> Option<String> {
    let patterns = ["remove-after:", "remove after:", "remove_after:"];
    let lower = line.to_lowercase();
    for p in patterns {
        if let Some(idx) = lower.find(p) {
            let after = line[idx + p.len()..].trim_start();
            let end = after
                .find([')', '\n', ',', '"'])
                .unwrap_or(after.len().min(30));
            let candidate = after[..end].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

fn parse_changelog(root: &Path, report: &mut DeprecationsReport) {
    let candidates = ["CHANGELOG.md", "CHANGES.md", "HISTORY.md"];
    for name in candidates {
        let path = root.join(name);
        if !path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        parse_changelog_content(&content, report);
        break;
    }
}

fn parse_changelog_content(content: &str, report: &mut DeprecationsReport) {
    let mut current_version: Option<String> = None;
    let mut current_section: Option<ChangelogKind> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("## [") {
            if let Some(end) = rest.find(']') {
                current_version = Some(rest[..end].to_string());
                current_section = None;
                continue;
            }
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            let ver = rest.split_whitespace().next().unwrap_or("").trim();
            if !ver.is_empty() {
                current_version = Some(ver.to_string());
                current_section = None;
                continue;
            }
        }

        if trimmed.starts_with("### ") {
            let heading = trimmed.trim_start_matches("### ").trim();
            current_section = match heading.to_lowercase().as_str() {
                s if s.contains("deprecated") => Some(ChangelogKind::Deprecated),
                s if s.contains("removed") => Some(ChangelogKind::Removed),
                s if s.contains("changed") || s.contains("breaking") => {
                    Some(ChangelogKind::Changed)
                }
                _ => None,
            };
            continue;
        }

        if let Some(kind) = current_section {
            if let Some(version) = &current_version {
                if let Some(bullet) = trimmed.strip_prefix("- ") {
                    if !bullet.is_empty() && bullet.len() > 5 {
                        report.changelog_deprecations.push(ChangelogEntry {
                            version: version.clone(),
                            kind,
                            text: bullet.trim().to_string(),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_rust_deprecated_attr() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("a.rs"),
            "#[deprecated(since = \"0.5.0\", note = \"use new_api instead\")]\npub fn old_api() {}\n",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].since.as_deref(), Some("0.5.0"));
    }

    #[test]
    fn detects_jsdoc_deprecated() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("a.ts"),
            "/** @deprecated use newFunc() instead */\nfunction oldFunc() {}\n",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(!report.items.is_empty());
        assert!(report.items[0].replacement.is_some());
    }

    #[test]
    fn detects_todo_remove_after() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("a.go"),
            "// TODO(remove-after: 2026-12-01): use v2 endpoint\nfunc oldEndpoint() {}\n",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(!report.items.is_empty());
        assert!(matches!(report.items[0].severity, Severity::Critical));
        assert_eq!(report.items[0].remove_after.as_deref(), Some("2026-12-01"));
    }

    #[test]
    fn parses_changelog_deprecated_section() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("CHANGELOG.md"),
            r#"# Changelog

## [1.0.0] - 2026-01-01

### Deprecated

- `oldFunction()` will be removed in 2.0, use `newFunction()` instead
- The `--old-flag` CLI option is deprecated in favor of `--new-flag`

### Removed

- `veryOldFunction()` was removed after being deprecated since 0.9
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.changelog_deprecations.len(), 3);
        assert!(matches!(
            report.changelog_deprecations[0].kind,
            ChangelogKind::Deprecated
        ));
    }

    #[test]
    fn extract_replacement_from_comment() {
        assert_eq!(
            extract_replacement("// deprecated, use new_api instead"),
            Some("new_api instead".to_string())
        );
    }

    #[test]
    fn empty_when_no_deprecations() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.rs"), "pub fn ok() {}\n").unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.total, 0);
    }
}
