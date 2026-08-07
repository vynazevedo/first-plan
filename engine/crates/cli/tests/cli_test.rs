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

// macOS runners no GitHub Actions sao flaky pra esse teste:
// daemon imprime 'starting' mas nao consegue escrever pid file em <10s
// (cold start de processo + tokio runtime + bind socket fica muito acima
// do esperado no runner compartilhado). Funcionalidade roda 100% em
// macOS local - skip aqui e cobre via Linux + smoke manual em Mac.
#[cfg(target_os = "linux")]
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
    for _ in 0..300 {
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
            "daemon failed to come up within 30s\nlast status: {}\nstderr: {}",
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

#[test]
fn generate_lists_all_adapters() {
    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["generate", "--list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let arr = parsed.as_array().expect("expected array of adapters");
    assert_eq!(arr.len(), 5, "expected 5 adapters, got {}", arr.len());
    let names: Vec<String> = arr
        .iter()
        .map(|a| a["name"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(names.contains(&"codex".to_string()));
    assert!(names.contains(&"cursor".to_string()));
    assert!(names.contains(&"copilot".to_string()));
    assert!(names.contains(&"cline".to_string()));
    assert!(names.contains(&"generic".to_string()));
}

#[test]
fn generate_codex_without_ir_falls_back_gracefully() {
    let tmp = TempDir::new().unwrap();
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "generate",
            "--tool",
            "codex",
            "--root",
            tmp.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();

    let agents_md = tmp.path().join("AGENTS.md");
    assert!(agents_md.exists(), "AGENTS.md should be created");
    let content = fs::read_to_string(&agents_md).unwrap();
    assert!(content.contains("first-plan-engine"));
    assert!(
        content.contains("No `.first-plan/`") || content.contains("run `first-plan-engine init`")
    );
}

#[test]
fn generate_all_creates_files_for_every_adapter() {
    let tmp = TempDir::new().unwrap();
    let ir_dir = tmp.path().join(".first-plan").join("02-conventions");
    fs::create_dir_all(&ir_dir).unwrap();
    fs::write(
        ir_dir.join("naming.md"),
        "# Naming\n\nUse snake_case for files.\n",
    )
    .unwrap();

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "generate",
            "--tool",
            "all",
            "--root",
            tmp.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();

    assert!(tmp.path().join("AGENTS.md").exists());
    assert!(tmp.path().join(".cursorrules").exists());
    assert!(tmp
        .path()
        .join(".cursor/rules/first-plan-context.mdc")
        .exists());
    assert!(tmp.path().join(".github/copilot-instructions.md").exists());
    assert!(tmp.path().join(".clinerules").exists());
    assert!(tmp.path().join("CONVENTIONS.md").exists());
}

#[test]
fn generate_cursor_produces_valid_mdc_frontmatter() {
    let tmp = TempDir::new().unwrap();
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "generate",
            "--tool",
            "cursor",
            "--root",
            tmp.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();

    let mdc = tmp.path().join(".cursor/rules/first-plan-context.mdc");
    assert!(mdc.exists());
    let content = fs::read_to_string(&mdc).unwrap();
    assert!(
        content.starts_with("---"),
        "mdc should start with frontmatter"
    );
    assert!(content.contains("description:"));
    assert!(content.contains("alwaysApply:"));
}

#[test]
fn init_list_layers_returns_expected_set() {
    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["init", "--list-layers", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["$schema"], "first-plan-init-layers-v1");
    let layers = parsed["layers"].as_array().unwrap();
    assert!(
        layers.len() >= 5,
        "esperava >= 5 layers, got {}",
        layers.len()
    );
    let names: Vec<String> = layers
        .iter()
        .map(|l| l["name"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(names.contains(&"mission/purpose".to_string()));
    assert!(names.contains(&"topology/stacks".to_string()));
}

#[test]
fn init_dry_run_collects_signals_without_llm_call() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("README.md"),
        "# Meu Projeto\n\nUm projeto de exemplo.\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"exemplo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "init",
            "--root",
            tmp.path().to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["$schema"], "first-plan-init-dry-v1");
    assert_eq!(parsed["dry_run"], true);
    let signals = &parsed["signals"];
    assert!(signals["readme"].is_string());
    let manifests = signals["manifests"].as_array().unwrap();
    assert!(
        manifests.iter().any(|m| m["path"] == "Cargo.toml"),
        "esperava Cargo.toml em manifests"
    );
    let stacks = signals["detected_stacks"].as_array().unwrap();
    assert!(
        stacks.iter().any(|s| s == "rust"),
        "esperava rust em detected_stacks"
    );
    let selected = parsed["layers_selected"].as_array().unwrap();
    assert!(!selected.is_empty(), "esperava layers selecionadas");
}

#[test]
fn init_dry_run_respects_layer_filter() {
    let tmp = TempDir::new().unwrap();

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "init",
            "--root",
            tmp.path().to_str().unwrap(),
            "--dry-run",
            "--layer",
            "mission/purpose",
            "--layer",
            "topology/stacks",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let selected = parsed["layers_selected"].as_array().unwrap();
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().any(|s| s == "mission/purpose"));
    assert!(selected.iter().any(|s| s == "topology/stacks"));
}

#[test]
fn llm_providers_lists_three_supported() {
    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(["llm", "providers", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["$schema"], "first-plan-llm-providers-v1");
    let providers = parsed["providers"].as_array().unwrap();
    assert_eq!(providers.len(), 3);
    let names: Vec<String> = providers
        .iter()
        .map(|p| p["name"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(names.contains(&"openai".to_string()));
    assert!(names.contains(&"anthropic".to_string()));
    assert!(names.contains(&"ollama".to_string()));
}

#[test]
fn multi_list_returns_empty_when_no_config() {
    let tmp = TempDir::new().unwrap();
    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "multi",
            "list",
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
    assert_eq!(parsed["$schema"], "first-plan-multi-list-v1");
    assert_eq!(parsed["total_repos"], 0);
    assert_eq!(parsed["repos"].as_array().unwrap().len(), 0);
}

#[test]
fn multi_register_then_list_persists_entry() {
    let tmp = TempDir::new().unwrap();
    let sibling = TempDir::new().unwrap();
    fs::write(sibling.path().join(".gitkeep"), "").unwrap();

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "multi",
            "register",
            "--name",
            "backend",
            "--path",
            sibling.path().to_str().unwrap(),
            "--tag",
            "rust",
            "--tag",
            "service",
            "--root",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let cfg_path = tmp.path().join(".first-plan/multi.yaml");
    assert!(cfg_path.exists(), "multi.yaml deveria existir");
    let content = fs::read_to_string(&cfg_path).unwrap();
    assert!(
        content.contains("name: backend"),
        "yaml deve conter 'name: backend'"
    );

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "multi",
            "list",
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
    assert_eq!(parsed["total_repos"], 1);
    let repo = &parsed["repos"][0];
    assert_eq!(repo["name"], "backend");
    let tags = repo["tags"].as_array().unwrap();
    assert!(tags.iter().any(|t| t == "rust"));
    assert!(tags.iter().any(|t| t == "service"));
    assert_eq!(repo["exists"], true);
}

#[test]
fn multi_register_rejects_duplicate_name() {
    let tmp = TempDir::new().unwrap();
    let sibling = TempDir::new().unwrap();
    let base = [
        "multi",
        "register",
        "--name",
        "dup",
        "--path",
        sibling.path().to_str().unwrap(),
        "--root",
        tmp.path().to_str().unwrap(),
    ];
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(base)
        .assert()
        .success();
    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args(base)
        .assert()
        .failure()
        .stderr(predicate::str::contains("já registrado"));
}

#[test]
fn multi_aggregate_produces_overview_markdown() {
    let root = TempDir::new().unwrap();
    let sibling = TempDir::new().unwrap();

    let mission_dir = sibling.path().join(".first-plan/00-mission");
    fs::create_dir_all(&mission_dir).unwrap();
    fs::write(
        mission_dir.join("purpose.md"),
        "---\nsection: mission/purpose\n---\n\n# Purpose\n\nEste repo faz coisas importantes.\n",
    )
    .unwrap();

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "multi",
            "register",
            "--name",
            "svc",
            "--path",
            sibling.path().to_str().unwrap(),
            "--root",
            root.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "multi",
            "aggregate",
            "--root",
            root.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let overview = root.path().join(".first-plan/multi/OVERVIEW.md");
    assert!(overview.exists(), "OVERVIEW.md deveria ter sido criado");
    let content = fs::read_to_string(&overview).unwrap();
    assert!(content.contains("section: multi/overview"));
    assert!(content.contains("## svc"));
    assert!(content.contains("Este repo faz coisas importantes"));
    assert!(content.contains("| svc |"));
}

fn write_openapi(dir: &Path, body: &str) {
    fs::write(dir.join("openapi.yaml"), body).unwrap();
}

const OPENAPI_BEFORE: &str = "openapi: 3.0.0
info:
  title: Users API
  version: 1.0.0
paths:
  /users:
    get:
      operationId: listUsers
  /legacy:
    delete:
      operationId: dropLegacy
";

const OPENAPI_AFTER_BREAKING: &str = "openapi: 3.0.0
info:
  title: Users API
  version: 1.1.0
paths:
  /users:
    get:
      operationId: fetchUsers
  /users/{id}/roles:
    post:
      operationId: assignRole
";

#[test]
fn contracts_snapshot_creates_json_file_with_endpoints() {
    let tmp = TempDir::new().unwrap();
    write_openapi(tmp.path(), OPENAPI_BEFORE);
    let out = tmp.path().join("snapshot.json");

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "contracts",
            "snapshot",
            "--root",
            tmp.path().to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();

    assert!(out.exists());
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    let endpoints = parsed["openapi"]["endpoints"].as_array().unwrap();
    assert_eq!(endpoints.len(), 2);
}

#[test]
fn contracts_diff_detects_breaking_and_non_breaking() {
    let before_dir = TempDir::new().unwrap();
    let after_dir = TempDir::new().unwrap();
    write_openapi(before_dir.path(), OPENAPI_BEFORE);
    write_openapi(after_dir.path(), OPENAPI_AFTER_BREAKING);

    let before_snap = before_dir.path().join("snap.json");
    let after_snap = after_dir.path().join("snap.json");

    for (dir, snap) in [(&before_dir, &before_snap), (&after_dir, &after_snap)] {
        Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args([
                "contracts",
                "snapshot",
                "--root",
                dir.path().to_str().unwrap(),
                "--out",
                snap.to_str().unwrap(),
            ])
            .assert()
            .success();
    }

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "contracts",
            "diff",
            "--before",
            before_snap.to_str().unwrap(),
            "--after",
            after_snap.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["summary"]["total_changes"], 3);
    assert_eq!(parsed["summary"]["breaking"], 2);
    assert_eq!(parsed["summary"]["non_breaking"], 1);
    let removed = parsed["openapi"]["removed"].as_array().unwrap();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0]["path"], "/legacy");
    assert_eq!(removed[0]["is_breaking"], true);
    let modified = parsed["openapi"]["modified"].as_array().unwrap();
    assert_eq!(modified.len(), 1);
    assert_eq!(modified[0]["path"], "/users");
    assert_eq!(modified[0]["is_breaking"], true);
}

#[test]
fn contracts_diff_fail_on_breaking_returns_nonzero() {
    let before_dir = TempDir::new().unwrap();
    let after_dir = TempDir::new().unwrap();
    write_openapi(before_dir.path(), OPENAPI_BEFORE);
    write_openapi(after_dir.path(), OPENAPI_AFTER_BREAKING);
    let before_snap = before_dir.path().join("snap.json");
    let after_snap = after_dir.path().join("snap.json");

    for (dir, snap) in [(&before_dir, &before_snap), (&after_dir, &after_snap)] {
        Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args([
                "contracts",
                "snapshot",
                "--root",
                dir.path().to_str().unwrap(),
                "--out",
                snap.to_str().unwrap(),
            ])
            .assert()
            .success();
    }

    Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "contracts",
            "diff",
            "--before",
            before_snap.to_str().unwrap(),
            "--after",
            after_snap.to_str().unwrap(),
            "--fail-on-breaking",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("breaking change"));
}

#[test]
fn multi_contracts_check_flags_breaking_repos() {
    let main = TempDir::new().unwrap();
    let clean_repo = TempDir::new().unwrap();
    let breaking_repo = TempDir::new().unwrap();

    for repo in [&clean_repo, &breaking_repo] {
        write_openapi(repo.path(), OPENAPI_BEFORE);
        let snap = repo.path().join(".first-plan/12-contracts/snapshot.json");
        Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args([
                "contracts",
                "snapshot",
                "--root",
                repo.path().to_str().unwrap(),
                "--out",
                snap.to_str().unwrap(),
            ])
            .assert()
            .success();
    }

    write_openapi(breaking_repo.path(), OPENAPI_AFTER_BREAKING);

    for (name, repo) in [("clean", &clean_repo), ("api", &breaking_repo)] {
        Command::cargo_bin("first-plan-engine")
            .unwrap()
            .args([
                "multi",
                "register",
                "--name",
                name,
                "--path",
                repo.path().to_str().unwrap(),
                "--root",
                main.path().to_str().unwrap(),
            ])
            .assert()
            .success();
    }

    let out = Command::cargo_bin("first-plan-engine")
        .unwrap()
        .args([
            "multi",
            "contracts-check",
            "--root",
            main.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["$schema"], "first-plan-multi-contracts-check-v1");
    assert_eq!(parsed["total_repos"], 2);
    assert_eq!(parsed["checked"], 2);
    assert_eq!(parsed["total_breaking"], 2);
    let repos = parsed["repos"].as_array().unwrap();
    let api = repos.iter().find(|r| r["name"] == "api").unwrap();
    assert_eq!(api["status"], "breaking");
    let clean = repos.iter().find(|r| r["name"] == "clean").unwrap();
    assert_eq!(clean["status"], "clean");
}
