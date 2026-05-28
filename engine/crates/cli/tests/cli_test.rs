//! Integration tests that exercise the compiled binary end to end.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn init_test_repo(dir: &Path) {
    let run = |args: &[&str]| {
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git available");
        assert!(status.success(), "git {:?} failed", args);
    };

    run(&["init", "--initial-branch=main"]);
    run(&["config", "user.email", "test@example.com"]);
    run(&["config", "user.name", "test"]);

    // 5 commits touching a.txt + b.txt together
    for i in 0..5 {
        fs::write(dir.join("a.txt"), format!("a v{}", i)).unwrap();
        fs::write(dir.join("b.txt"), format!("b v{}", i)).unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", &format!("c{}", i)]);
    }

    // 2 commits touching only a.txt (so total_a > total_b but ratio still high)
    for i in 5..7 {
        fs::write(dir.join("a.txt"), format!("a v{}", i)).unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", &format!("c{}", i)]);
    }
}

#[test]
fn cochange_detects_pair_in_real_git_repo() {
    let tmp = TempDir::new().unwrap();
    init_test_repo(tmp.path());

    let out = tmp.path().join("cc.json");
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "cochange",
            "--repo",
            tmp.path().to_str().unwrap(),
            "--since",
            "1",
            "--min-occurrences",
            "5",
            "--min-ratio",
            "0.5",
            "--output-json",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = fs::read_to_string(&out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["$schema"], "first-plan-cochange-v1");
    let pairs = parsed["pairs"].as_array().unwrap();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0]["file_a"], "a.txt");
    assert_eq!(pairs[0]["file_b"], "b.txt");
    assert_eq!(pairs[0]["shared_commits"], 5);
    assert_eq!(pairs[0]["total_a"], 7);
    assert_eq!(pairs[0]["total_b"], 5);
}

#[test]
fn cochange_fails_on_non_git_directory() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "cochange",
            "--repo",
            tmp.path().to_str().unwrap(),
            "--output-json",
            "-",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));
}

#[test]
fn hash_processes_files_via_args() {
    let tmp = TempDir::new().unwrap();
    let f1 = tmp.path().join("a.txt");
    let f2 = tmp.path().join("b.txt");
    fs::write(&f1, "alpha").unwrap();
    fs::write(&f2, "beta").unwrap();

    let out = tmp.path().join("h.json");
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "hash",
            "--paths",
            f1.to_str().unwrap(),
            "--paths",
            f2.to_str().unwrap(),
            "--output-json",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = fs::read_to_string(&out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["$schema"], "first-plan-hash-v1");
    assert_eq!(parsed["algorithm"], "xxh3_64");
    assert_eq!(parsed["files"].as_object().unwrap().len(), 2);
}

#[test]
fn hash_fails_with_no_paths() {
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["hash", "--output-json", "-"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no paths provided"));
}

#[test]
fn lsp_status_runs_and_returns_json() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[package]\nname='x'\n").unwrap();

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "lsp",
            "status",
            "--root",
            tmp.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(parsed["engine_version"].is_string());
    assert!(parsed["servers"].is_array());
    assert_eq!(parsed["servers"].as_array().unwrap().len(), 8);

    let needs = parsed["project_needs"].as_array().unwrap();
    assert!(needs.iter().any(|v| v == "rust-analyzer"));
}

#[test]
fn lsp_refs_falls_back_to_grep_when_no_lsp() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.rs"),
        "fn target() {}\nfn other() { target(); }\n",
    )
    .unwrap();

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "lsp",
            "refs",
            "--file",
            tmp.path().join("a.rs").to_str().unwrap(),
            "--line",
            "0",
            "--col",
            "3",
            "--root",
            tmp.path().to_str().unwrap(),
            "--no-lsp",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["op"], "references");
    assert_eq!(parsed["used_fallback"], true);
    let data = parsed["data"].as_array().unwrap();
    assert!(
        data.len() >= 2,
        "esperava >= 2 referencias, got {}",
        data.len()
    );
}

#[cfg(unix)]
#[test]
fn lsp_daemon_status_when_not_running() {
    use std::thread::sleep;
    use std::time::Duration;

    let _ = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["lsp", "daemon", "stop"])
        .assert();

    // Aguarda daemon de outros testes encerrar completamente (CI compartilha runtime dir).
    for _ in 0..30 {
        let out = Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args(["lsp", "daemon", "status", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
        if parsed["running"] == false {
            assert!(parsed["socket_path"].is_string());
            return;
        }
        sleep(Duration::from_millis(100));
    }
    panic!("daemon ainda rodando apos 3s de espera");
}

#[cfg(unix)]
#[test]
fn lsp_daemon_start_then_status_then_stop() {
    use std::process::{Command as StdCommand, Stdio};
    use std::thread::sleep;
    use std::time::Duration;

    let _ = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["lsp", "daemon", "stop"])
        .assert();

    let bin = assert_cmd::cargo::cargo_bin("first-plan-engine");
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[package]\nname='x'\n").unwrap();

    let mut child = StdCommand::new(&bin)
        .args([
            "lsp",
            "daemon",
            "start",
            "--root",
            tmp.path().to_str().unwrap(),
            "--idle-minutes",
            "1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daemon");

    let mut running = false;
    let mut last_status = String::new();
    for _ in 0..100 {
        sleep(Duration::from_millis(100));
        let out = Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args(["lsp", "daemon", "status", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        last_status = String::from_utf8_lossy(&out).into_owned();
        let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
        if parsed["running"] == true {
            running = true;
            assert!(parsed["pid"].as_u64().unwrap() > 0);
            assert!(parsed["uptime_seconds"].is_number());
            break;
        }
    }
    if !running {
        let _ = child.kill();
        let output = child.wait_with_output().ok();
        let stderr = output
            .as_ref()
            .map(|o| String::from_utf8_lossy(&o.stderr).into_owned())
            .unwrap_or_default();
        panic!(
            "daemon failed to come up within 10s\nlast status: {}\nstderr: {}",
            last_status, stderr
        );
    }

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["lsp", "daemon", "stop"])
        .assert()
        .success();

    let mut stopped = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(100));
        let out = Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args(["lsp", "daemon", "status", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
        if parsed["running"] == false {
            stopped = true;
            break;
        }
    }
    assert!(stopped, "daemon nao parou apos 3s de stop");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn lsp_wsymbols_fallback_finds_function_definition() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("lib.rs"), "pub fn unique_func_name() {}\n").unwrap();

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "lsp",
            "wsymbols",
            "--query",
            "unique_func",
            "--root",
            tmp.path().to_str().unwrap(),
            "--no-lsp",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["op"], "workspaceSymbol");
    assert_eq!(parsed["used_fallback"], true);
    let names: Vec<String> = parsed["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(names.iter().any(|n| n == "unique_func_name"));
}
