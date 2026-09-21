use super::*;
use std::fs;
use tempfile::TempDir;

fn fixture() -> (TempDir, TempDir, Policy) {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::create_dir(root.path().join(".first-plan")).unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(root.path().join("src/auth.txt"), "tenant isolation").unwrap();
    fs::write(root.path().join("test.txt"), "reviewed verifier").unwrap();
    let registry = Registry {
        schema_version: 1,
        rules: vec![Rule {
            id: "tenant-isolation".into(),
            requirement: "Users may only access their own tenant".into(),
            owner: "security".into(),
            keywords: vec!["authorization".into()],
            inputs: vec!["src".into()],
            verification_files: vec!["test.txt".into()],
            verifier: Verifier::Test {
                command: vec![
                    std::env::current_exe().unwrap().to_string_lossy().into(),
                    "--list".into(),
                ],
                timeout_seconds: 5,
            },
        }],
    };
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    let policy = candidate_policy(root.path()).unwrap();
    fs::write(
        outside.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    (root, outside, policy)
}

#[test]
fn binds_rules_and_verifiers_without_freezing_implementation_changes() {
    let (root, _, policy) = fixture();
    fs::write(root.path().join("src/auth.txt"), "new implementation").unwrap();
    assert_eq!(review(root.path(), &policy).unwrap().status, "authorized");
    fs::write(root.path().join("test.txt"), "weakened verifier").unwrap();
    let diff = review(root.path(), &policy).unwrap();
    assert_eq!(diff.status, "review_required");
    assert_eq!(diff.changed_verification_files, vec!["test.txt"]);
    assert!(require_authorized(root.path(), &policy).is_err());
}

#[test]
fn rule_removal_and_requirement_changes_need_review() {
    let (root, _, policy) = fixture();
    let mut registry = load(root.path()).unwrap();
    registry.rules[0].requirement = "weaker rule".into();
    registry.rules[0].id = "replacement".into();
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    let diff = review(root.path(), &policy).unwrap();
    assert!(diff.registry_changed);
    assert_eq!(diff.removed_rules, vec!["tenant-isolation"]);
    assert_eq!(diff.changed_rules.len(), 2);
    assert_eq!(diff.added_rules, vec!["replacement"]);
}

#[test]
fn no_policy_in_project_and_no_execution_on_drift() {
    let (root, outside, policy) = fixture();
    let inner = root.path().join("policy.json");
    fs::write(&inner, serde_json::to_vec(&policy).unwrap()).unwrap();
    assert!(external_policy(root.path(), &inner).is_err());
    fs::write(root.path().join("test.txt"), "changed").unwrap();
    let out = outside.path().join("report.json");
    assert!(run(root.path(), &outside.path().join("policy.json"), &out).is_err());
    assert!(!out.exists());
}

#[test]
fn reports_detect_new_inputs_and_tampered_logs() {
    let (root, outside, _) = fixture();
    let policy = outside.path().join("policy.json");
    let out = outside.path().join("report.json");
    let report = run(root.path(), &policy, &out).unwrap();
    assert_eq!(report.status, "passed");
    assert_eq!(report.results[0].status, "tests_passed");
    assert_eq!(check_report(root.path(), &out, &policy).unwrap(), "passed");
    fs::write(root.path().join("src/new.txt"), "new unchecked source").unwrap();
    assert_eq!(check_report(root.path(), &out, &policy).unwrap(), "stale");
    fs::remove_file(root.path().join("src/new.txt")).unwrap();
    fs::write(outside.path().join(&report.results[0].log), "tampered log").unwrap();
    assert_eq!(check_report(root.path(), &out, &policy).unwrap(), "stale");
    assert!(run(root.path(), &policy, &out).is_err());
}

#[test]
fn context_includes_explicit_obligations_even_with_small_budget() {
    let (root, _, _) = fixture();
    let pack =
        crate::context::build_for_paths(root.path(), "unrelated", 256, &["src/auth.txt".into()])
            .unwrap();
    assert_eq!(pack.applicable_rules.len(), 1);
    assert_eq!(pack.applicable_rules[0].match_reason, "declared_path");
    let lexical = applicable(root.path(), "authorization", &[]).unwrap();
    assert_eq!(lexical[0].match_reason, "lexical_candidate");
    assert!(applicable(root.path(), "unrelated", &[])
        .unwrap()
        .is_empty());
    assert!(applicable(root.path(), "query", &["../outside".into()]).is_err());
}

#[test]
fn invalid_registry_missing_inputs_and_tools_never_pass() {
    let (root, _, _) = fixture();
    let mut registry = load(root.path()).unwrap();
    registry.rules[0].inputs = vec!["missing".into()];
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    assert!(candidate_policy(root.path()).is_err());
    registry.rules[0].inputs = vec!["src".into()];
    registry.rules[0].verifier = Verifier::Test {
        command: vec!["nonexistent-first-plan-verifier-000".into()],
        timeout_seconds: 1,
    };
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    assert!(candidate_policy(root.path()).is_err());
    registry.rules.push(registry.rules[0].clone());
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    assert!(load(root.path()).is_err());
    for path in [
        "../escape",
        "/absolute",
        "src/../secret",
        ".git/config",
        "C:\\secret",
    ] {
        assert!(relative(path).is_err());
    }
}

#[test]
#[cfg(unix)]
fn symlinks_cannot_import_outside_data() {
    let (root, outside, _) = fixture();
    fs::write(outside.path().join("secret"), "private").unwrap();
    std::os::unix::fs::symlink(outside.path().join("secret"), root.path().join("src/link"))
        .unwrap();
    assert!(candidate_policy(root.path()).is_err());
}

#[test]
#[ignore = "subprocess fixture"]
fn slow_child() {
    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[test]
#[ignore = "subprocess fixture"]
fn failing_child() {
    panic!("deliberate failing test");
}

#[test]
fn timeout_and_failure_are_distinguished_from_pass() {
    for (child, timeout, expected) in [
        ("slow_child", 1, "inconclusive"),
        ("failing_child", 5, "failed"),
    ] {
        let (root, outside, _) = fixture();
        let mut registry = load(root.path()).unwrap();
        registry.rules[0].verifier = Verifier::Test {
            command: vec![
                std::env::current_exe().unwrap().to_string_lossy().into(),
                "--exact".into(),
                format!("rules::tests::{child}"),
                "--ignored".into(),
            ],
            timeout_seconds: timeout,
        };
        fs::write(
            root.path().join(REGISTRY),
            serde_yaml::to_string(&registry).unwrap(),
        )
        .unwrap();
        let policy = candidate_policy(root.path()).unwrap();
        let p = outside.path().join("policy.json");
        fs::write(&p, serde_json::to_vec(&policy).unwrap()).unwrap();
        let r = run(root.path(), &p, &outside.path().join("report.json")).unwrap();
        assert_eq!(r.status, expected);
        assert_eq!(r.results[0].status, expected);
    }
}

#[test]
fn formal_adapter_rejects_zero_checks_wrong_harness_and_unwinding() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("kani.json");
    let mut data = serde_json::json!({"metadata":{"version":"1.0","kani_version":"0.68.0","target":"x86_64-unknown-linux-gnu"},
        "verification_results":{"summary":{"status":"completed","total_harnesses":1,"executed":1,"successful":1,"failed":0},
        "results":[{"harness_id":"my::proof","status":"Success","checks":[{"category":"assertion","status":"Success"}]}]}});
    fs::write(&file, data.to_string()).unwrap();
    assert_eq!(
        execution::kani_status(&file, "my::proof", Some(0)).unwrap(),
        "formally_verified"
    );
    assert!(execution::kani_status(&file, "other", Some(0)).is_err());
    data["verification_results"]["results"][0]["checks"] = serde_json::json!([]);
    fs::write(&file, data.to_string()).unwrap();
    assert!(execution::kani_status(&file, "my::proof", Some(0)).is_err());
    data["verification_results"]["results"][0]["checks"] = serde_json::json!([{"category":"assertion","status":"Success"},{"category":"unwind","status":"Failure"}]);
    fs::write(&file, data.to_string()).unwrap();
    assert_eq!(
        execution::kani_status(&file, "my::proof", Some(1)).unwrap(),
        "inconclusive"
    );
}

#[test]
#[ignore = "subprocess fixture"]
fn mutating_child() {
    fs::write("src/auth.txt", "changed by verifier").unwrap();
}

#[test]
fn source_changes_during_execution_preserve_partial_evidence_and_stop_checks() {
    let (root, outside, _) = fixture();
    let mut registry = load(root.path()).unwrap();
    let mut second = registry.rules[0].clone();
    second.id = "second".into();
    registry.rules[0].verifier = Verifier::Test {
        command: vec![
            std::env::current_exe().unwrap().to_string_lossy().into(),
            "--exact".into(),
            "rules::tests::mutating_child".into(),
            "--ignored".into(),
        ],
        timeout_seconds: 5,
    };
    registry.rules.push(second);
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    let policy = candidate_policy(root.path()).unwrap();
    let p = outside.path().join("policy.json");
    fs::write(&p, serde_json::to_vec(&policy).unwrap()).unwrap();
    let out = outside.path().join("partial.json");
    let report = run(root.path(), &p, &out).unwrap();
    assert_eq!(report.status, "stale");
    assert_eq!(report.results[0].status, "tests_passed");
    assert_eq!(report.results[1].assurance, "not_executed");
    assert!(out.is_file());
}

#[test]
fn replacement_policy_cannot_make_a_report_pass_the_original_approval() {
    let (root, outside, _) = fixture();
    let original = outside.path().join("policy.json");
    let mut registry = load(root.path()).unwrap();
    registry.rules[0].requirement = "weaker requirement".into();
    fs::write(
        root.path().join(REGISTRY),
        serde_yaml::to_string(&registry).unwrap(),
    )
    .unwrap();
    let replacement = outside.path().join("replacement.json");
    fs::write(
        &replacement,
        serde_json::to_vec(&candidate_policy(root.path()).unwrap()).unwrap(),
    )
    .unwrap();
    let out = outside.path().join("replacement-report.json");
    assert_eq!(
        run(root.path(), &replacement, &out).unwrap().status,
        "passed"
    );
    assert_eq!(check_report(root.path(), &out, &original).unwrap(), "stale");
}
