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
