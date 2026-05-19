use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompressedOutput {
    pub tool: String,
    pub raw_bytes: usize,
    pub compressed_bytes: usize,
    pub savings_pct: f32,
    pub exit_code: i32,
    pub output: String,
}

pub fn run_and_compress(tool: &str, args: &[String]) -> Result<CompressedOutput> {
    let (cmd, fallback_args) = resolve_command(tool);
    let final_args: Vec<String> = if fallback_args.is_empty() {
        args.to_vec()
    } else {
        let mut combined: Vec<String> = fallback_args.iter().map(|s| s.to_string()).collect();
        combined.extend_from_slice(args);
        combined
    };

    let output = Command::new(&cmd)
        .args(&final_args)
        .output()
        .with_context(|| format!("spawn {} {:?}", cmd, final_args))?;

    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined_raw = if stderr.is_empty() {
        raw.clone()
    } else {
        format!("{}\n{}", raw, stderr)
    };

    let compressed = compress(tool, &raw, &stderr);
    let raw_bytes = combined_raw.len();
    let compressed_bytes = compressed.len();
    let savings_pct = if raw_bytes == 0 {
        0.0
    } else {
        (1.0 - compressed_bytes as f32 / raw_bytes as f32) * 100.0
    };

    Ok(CompressedOutput {
        tool: tool.to_string(),
        raw_bytes,
        compressed_bytes,
        savings_pct,
        exit_code: output.status.code().unwrap_or(-1),
        output: compressed,
    })
}

fn resolve_command(tool: &str) -> (String, Vec<&'static str>) {
    match tool {
        "git-status" => ("git".into(), vec!["status", "--short"]),
        "git-log" => ("git".into(), vec!["log", "--oneline", "-n", "20"]),
        "git-diff" => ("git".into(), vec!["diff"]),
        "git-diff-stat" => ("git".into(), vec!["diff", "--stat"]),
        "git-branch" => ("git".into(), vec!["branch", "-a"]),
        "find" => ("find".into(), vec![]),
        "grep" => ("grep".into(), vec![]),
        "rg" => ("rg".into(), vec![]),
        "ls" => ("ls".into(), vec!["-la"]),
        "cargo-check" => ("cargo".into(), vec!["check", "--message-format=short"]),
        "cargo-test" => ("cargo".into(), vec!["test"]),
        "cargo-metadata" => (
            "cargo".into(),
            vec!["metadata", "--format-version=1", "--no-deps"],
        ),
        "npm-test" => ("npm".into(), vec!["test"]),
        "go-build" => ("go".into(), vec!["build", "./..."]),
        "go-test" => ("go".into(), vec!["test", "./..."]),
        _ => (tool.to_string(), vec![]),
    }
}

pub fn compress(tool: &str, stdout: &str, stderr: &str) -> String {
    match tool {
        "git-status" => compress_git_status(stdout),
        "git-log" => compress_git_log(stdout),
        "git-diff" => compress_git_diff(stdout),
        "git-diff-stat" => compress_git_diff_stat(stdout),
        "git-branch" => compress_git_branch(stdout),
        "find" => compress_find(stdout),
        "grep" | "rg" => compress_grep(stdout),
        "ls" => compress_ls(stdout),
        "cargo-check" | "cargo-test" | "go-build" | "go-test" | "npm-test" => {
            compress_test_output(stdout, stderr)
        }
        "cargo-metadata" => compress_cargo_metadata(stdout),
        _ => format!("{}\n{}", stdout, stderr).trim().to_string(),
    }
}

fn compress_git_status(stdout: &str) -> String {
    let mut modified = Vec::new();
    let mut added = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();
    let mut untracked = Vec::new();

    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }
        let status = &line[..2];
        let path = line[3..].trim();
        match status.trim() {
            "M" | "MM" | "AM" => modified.push(path),
            "A" => added.push(path),
            "D" => deleted.push(path),
            "R" => renamed.push(path),
            "??" => untracked.push(path),
            _ => {}
        }
    }

    let mut out = String::new();
    if !modified.is_empty() {
        out.push_str(&format!("M({}): {}\n", modified.len(), modified.join(" ")));
    }
    if !added.is_empty() {
        out.push_str(&format!("A({}): {}\n", added.len(), added.join(" ")));
    }
    if !deleted.is_empty() {
        out.push_str(&format!("D({}): {}\n", deleted.len(), deleted.join(" ")));
    }
    if !renamed.is_empty() {
        out.push_str(&format!("R({}): {}\n", renamed.len(), renamed.join(" ")));
    }
    if !untracked.is_empty() {
        let preview: Vec<&str> = untracked.iter().take(10).copied().collect();
        let tail = if untracked.len() > 10 {
            format!(" (+{} more)", untracked.len() - 10)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "??({}): {}{}\n",
            untracked.len(),
            preview.join(" "),
            tail
        ));
    }
    if out.is_empty() {
        "clean".to_string()
    } else {
        out.trim_end().to_string()
    }
}

fn compress_git_log(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return "no commits".to_string();
    }
    let show_n = lines.len().min(15);
    let mut out = lines
        .iter()
        .take(show_n)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    if lines.len() > show_n {
        out.push_str(&format!("\n... +{} earlier commits", lines.len() - show_n));
    }
    out
}

fn compress_git_diff(stdout: &str) -> String {
    let mut files: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut current_file: Option<String> = None;
    let mut total_lines = 0;

    for line in stdout.lines() {
        total_lines += 1;
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = Some(rest.to_string());
            files.entry(rest.to_string()).or_insert((0, 0));
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            if current_file.is_none() {
                current_file = Some(rest.to_string());
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            if let Some(f) = &current_file {
                files.entry(f.clone()).or_insert((0, 0)).0 += 1;
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            if let Some(f) = &current_file {
                files.entry(f.clone()).or_insert((0, 0)).1 += 1;
            }
        }
    }

    if files.is_empty() {
        return "no changes".to_string();
    }

    let mut out = String::new();
    out.push_str(&format!(
        "diff: {} files, raw {} lines\n",
        files.len(),
        total_lines
    ));
    for (path, (add, del)) in &files {
        out.push_str(&format!("{} +{}/-{}\n", path, add, del));
    }
    out.trim_end().to_string()
}

fn compress_git_diff_stat(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if lines.len() <= 20 {
        return stdout.trim().to_string();
    }
    let mut out: Vec<&str> = lines.iter().take(18).copied().collect();
    out.push("...");
    if let Some(last) = lines.last() {
        out.push(*last);
    }
    out.join("\n")
}

fn compress_git_branch(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();
    let current = lines
        .iter()
        .find(|l| l.starts_with('*'))
        .map(|l| l.trim_start_matches('*').trim())
        .unwrap_or("?");
    let local: Vec<&str> = lines
        .iter()
        .filter(|l| !l.starts_with('*') && !l.contains("remotes/"))
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let remote_count = lines.iter().filter(|l| l.contains("remotes/")).count();

    let mut out = format!("current: {}", current);
    if !local.is_empty() {
        out.push_str(&format!("\nlocal ({}): {}", local.len(), local.join(" ")));
    }
    if remote_count > 0 {
        out.push_str(&format!("\nremote: {} branches", remote_count));
    }
    out
}

fn compress_find(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    let total = lines.len();
    if total <= 50 {
        return stdout.trim().to_string();
    }

    let mut by_dir: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for line in &lines {
        let dir = std::path::Path::new(line)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        by_dir.entry(dir).or_default().push(line);
    }

    let mut out = format!("found {} files in {} dirs\n", total, by_dir.len());
    for (dir, files) in by_dir.iter().take(20) {
        out.push_str(&format!(
            "{}: {} files (e.g. {})\n",
            dir,
            files.len(),
            files
                .iter()
                .take(3)
                .map(|f| {
                    std::path::Path::new(f)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| f.to_string())
                })
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if by_dir.len() > 20 {
        out.push_str(&format!("... +{} more dirs", by_dir.len() - 20));
    }
    out.trim_end().to_string()
}

fn compress_grep(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    let total = lines.len();
    if total <= 30 {
        return stdout.trim().to_string();
    }

    let mut by_file: BTreeMap<String, u32> = BTreeMap::new();
    for line in &lines {
        if let Some(colon) = line.find(':') {
            let file = &line[..colon];
            *by_file.entry(file.to_string()).or_insert(0) += 1;
        }
    }

    let mut out = format!("grep: {} matches across {} files\n", total, by_file.len());
    let mut sorted: Vec<(&String, &u32)> = by_file.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (file, count) in sorted.iter().take(15) {
        out.push_str(&format!("{}: {} matches\n", file, count));
    }
    if sorted.len() > 15 {
        out.push_str(&format!("... +{} more files", sorted.len() - 15));
    }
    out.trim_end().to_string()
}

fn compress_ls(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if lines.len() <= 20 {
        return stdout.trim().to_string();
    }

    let mut dirs = 0;
    let mut files = 0;
    let mut entries: Vec<&str> = Vec::new();
    for line in &lines {
        if line.starts_with('d') {
            dirs += 1;
        } else if line.starts_with('-') {
            files += 1;
        }
        entries.push(line);
    }

    let mut out = format!("ls: {} dirs, {} files\n", dirs, files);
    for line in entries.iter().take(15) {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!("... +{} more entries", lines.len() - 15));
    out
}

fn compress_test_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let lines: Vec<&str> = combined.lines().collect();

    let mut failures = Vec::new();
    let mut errors = Vec::new();
    let mut summary = None;

    for line in &lines {
        let lower = line.to_lowercase();
        if lower.contains("fail") || lower.contains("panic") || lower.starts_with("error[") {
            failures.push(*line);
        } else if lower.starts_with("error:") || lower.contains(": error:") {
            errors.push(*line);
        } else if lower.contains("test result:")
            || lower.contains("tests passed")
            || lower.contains("tests failed")
            || lower.contains("finished")
        {
            summary = Some(*line);
        }
    }

    let mut out = String::new();
    if let Some(s) = summary {
        out.push_str(&format!("{}\n", s));
    }
    if !failures.is_empty() {
        out.push_str(&format!("\nFAILURES ({}):\n", failures.len()));
        for f in failures.iter().take(20) {
            out.push_str(&format!("  {}\n", f));
        }
        if failures.len() > 20 {
            out.push_str(&format!("  ... +{} more failures\n", failures.len() - 20));
        }
    }
    if !errors.is_empty() {
        out.push_str(&format!("\nERRORS ({}):\n", errors.len()));
        for e in errors.iter().take(20) {
            out.push_str(&format!("  {}\n", e));
        }
        if errors.len() > 20 {
            out.push_str(&format!("  ... +{} more errors\n", errors.len() - 20));
        }
    }
    if out.is_empty() {
        return format!("ok ({} lines)", lines.len());
    }
    out.trim_end().to_string()
}

fn compress_cargo_metadata(stdout: &str) -> String {
    let parsed: serde_json::Value = match serde_json::from_str(stdout) {
        Ok(v) => v,
        Err(_) => return stdout.trim().to_string(),
    };

    let packages = parsed
        .get("packages")
        .and_then(|p| p.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);
    let workspace_members = parsed
        .get("workspace_members")
        .and_then(|w| w.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);
    let target_dir = parsed
        .get("target_directory")
        .and_then(|t| t.as_str())
        .unwrap_or("?");
    let workspace_root = parsed
        .get("workspace_root")
        .and_then(|w| w.as_str())
        .unwrap_or("?");

    let mut names = Vec::new();
    if let Some(pkgs) = parsed.get("packages").and_then(|p| p.as_array()) {
        for p in pkgs.iter().take(10) {
            if let Some(name) = p.get("name").and_then(|n| n.as_str()) {
                names.push(name);
            }
        }
    }

    format!(
        "cargo metadata\nworkspace: {}\ntarget: {}\nmembers: {}, packages: {}\nfirst names: {}",
        workspace_root,
        target_dir,
        workspace_members,
        packages,
        names.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_status_groups_by_type() {
        let mut lines = Vec::new();
        for i in 0..15 {
            lines.push(format!(" M src/path/to/very/deep/file_{}.rs", i));
        }
        for i in 0..5 {
            lines.push(format!("A  src/new_file_{}.rs", i));
        }
        for i in 0..20 {
            lines.push(format!("?? generated/file_{}.tmp", i));
        }
        let raw = lines.join("\n");
        let out = compress_git_status(&raw);
        assert!(out.contains("M(15)"));
        assert!(out.contains("A(5)"));
        assert!(out.contains("??(20)"));
        assert!(
            out.len() < raw.len(),
            "raw={} compressed={}",
            raw.len(),
            out.len()
        );
    }

    #[test]
    fn git_status_clean() {
        assert_eq!(compress_git_status(""), "clean");
    }

    #[test]
    fn git_log_truncates_long_history() {
        let lines: Vec<String> = (0..50)
            .map(|i| format!("abc{:04} commit {}", i, i))
            .collect();
        let raw = lines.join("\n");
        let out = compress_git_log(&raw);
        assert!(out.contains("+35 earlier commits"));
        assert!(out.lines().count() < 20);
    }

    #[test]
    fn git_diff_summarizes_by_file() {
        let raw = "--- a/foo.go\n+++ b/foo.go\n+added line 1\n+added line 2\n-removed line\n";
        let out = compress_git_diff(raw);
        assert!(out.contains("foo.go"));
        assert!(out.contains("+2/-1"));
    }

    #[test]
    fn find_groups_by_directory_when_long() {
        let mut paths = Vec::new();
        for i in 0..100 {
            paths.push(format!("./src/module_{}/file.rs", i % 5));
        }
        let raw = paths.join("\n");
        let out = compress_find(&raw);
        assert!(out.contains("100 files"));
        assert!(out.len() < raw.len() / 2);
    }

    #[test]
    fn find_preserves_short_output() {
        let raw = "./a.rs\n./b.rs\n./c.rs";
        let out = compress_find(raw);
        assert_eq!(out, raw);
    }

    #[test]
    fn grep_collapses_matches_by_file() {
        let mut lines = Vec::new();
        for i in 0..50 {
            lines.push(format!("src/main.rs:{}:hit", i));
            lines.push(format!("src/lib.rs:{}:hit", i));
        }
        let raw = lines.join("\n");
        let out = compress_grep(&raw);
        assert!(out.contains("100 matches"));
        assert!(out.contains("src/main.rs: 50 matches"));
        assert!(out.len() < raw.len());
    }

    #[test]
    fn test_output_extracts_failures_only() {
        let raw = "running 100 tests\nok 1\nok 2\nFAILED 3 - assertion\nok 4\ntest result: ok. 99 passed; 1 failed";
        let out = compress_test_output(raw, "");
        assert!(out.contains("test result"));
        assert!(out.contains("FAILED"));
        assert!(out.len() < raw.len());
    }

    #[test]
    fn unknown_tool_passthrough() {
        let out = compress("unknown-tool", "some output", "");
        assert!(out.contains("some output"));
    }
}
