use super::*;
use std::fs::File;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleResult {
    pub id: String,
    pub status: String,
    pub assurance: String,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub elapsed_ms: u128,
    pub detail: Option<String>,
    pub log: String,
    pub log_hash: String,
    pub artifact: Option<String>,
    pub artifact_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema_version: u32,
    pub status: String,
    pub generated_at: String,
    pub revision: Option<String>,
    pub inputs: BTreeMap<String, String>,
    pub policy: Policy,
    pub results: Vec<RuleResult>,
    pub limitations: Vec<String>,
}

struct Process(Child);
impl Process {
    fn terminate(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        {
            if let Some(root) = std::env::var_os("SystemRoot") {
                let _ = Command::new(PathBuf::from(root).join("System32/taskkill.exe"))
                    .args(["/PID", &self.0.id().to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn invoke(
    exe: &Path,
    args: &[String],
    cwd: &Path,
    log: &Path,
    timeout: u64,
) -> Result<(Option<ExitStatus>, Option<String>)> {
    let file = File::create(log)?;
    let mut command = Command::new(exe);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(file.try_clone()?)
        .stderr(file);
    // Avoid Python bytecode becoming a newly discovered input on verification runs.
    command.env("PYTHONDONTWRITEBYTECODE", "1");
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut process = Process(command.spawn()?);
    let start = Instant::now();
    loop {
        if let Some(status) = process.0.try_wait()? {
            return Ok((Some(status), None));
        }
        if start.elapsed() >= Duration::from_secs(timeout) {
            return Ok((None, Some("timeout".into())));
        }
        if std::fs::metadata(log)?.len() > 8_000_000 {
            return Ok((None, Some("log limit exceeded".into())));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(super) fn kani_status(path: &Path, harness: &str, code: Option<i32>) -> Result<&'static str> {
    ensure!(
        std::fs::metadata(path)?.len() <= 16_000_000,
        "Kani artifact too large"
    );
    let data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    ensure!(
        data["metadata"]["version"] == "1.0"
            && data["metadata"]["kani_version"] == "0.68.0"
            && data["metadata"]["target"] == "x86_64-unknown-linux-gnu",
        "unsupported Kani metadata"
    );
    let summary = &data["verification_results"]["summary"];
    ensure!(
        summary["status"] == "completed"
            && summary["total_harnesses"] == 1
            && summary["executed"] == 1,
        "Kani must complete exactly one harness"
    );
    let results = data["verification_results"]["results"]
        .as_array()
        .context("missing Kani results")?;
    ensure!(
        results.len() == 1 && results[0]["harness_id"] == harness,
        "unexpected Kani harness"
    );
    let checks = results[0]["checks"]
        .as_array()
        .context("missing Kani checks")?;
    ensure!(
        checks
            .iter()
            .any(|c| c["category"] == "assertion" && c["status"] == "Success")
            || checks
                .iter()
                .any(|c| c["category"] == "assertion" && c["status"] == "Failure"),
        "no executed assertions"
    );
    ensure!(
        checks.iter().all(|c| matches!(
            c["status"].as_str(),
            Some("Success" | "Failure" | "Unreachable")
        )),
        "inconclusive Kani checks"
    );
    let failures: Vec<_> = checks.iter().filter(|c| c["status"] == "Failure").collect();
    if code == Some(0)
        && results[0]["status"] == "Success"
        && summary["successful"] == 1
        && summary["failed"] == 0
        && failures.is_empty()
    {
        return Ok("formally_verified");
    }
    if code.is_some_and(|c| c != 0)
        && results[0]["status"] == "Failure"
        && !failures.is_empty()
        && failures.iter().all(|c| c["category"] == "assertion")
    {
        return Ok("failed");
    }
    Ok("inconclusive")
}

fn passed(status: &str) -> bool {
    matches!(status, "tests_passed" | "formally_verified")
}

pub fn run(root: &Path, policy_path: &Path, output: &Path) -> Result<Report> {
    let root = root.canonicalize()?;
    let policy = external_policy(&root, policy_path)?;
    require_authorized(&root, &policy)?;
    let registry = load(&root)?;
    let before = inputs(&root, &registry)?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let parent = parent.canonicalize()?;
    ensure!(
        !parent.starts_with(&root),
        "store verification reports outside the project"
    );
    ensure!(
        !output.exists(),
        "choose a new report path; existing reports are preserved"
    );
    let run_name = format!(
        "fpe-evidence-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let logs = parent.join(&run_name);
    std::fs::create_dir(&logs)?;
    let mut results = Vec::new();
    for rule in &registry.rules {
        // Re-check approval and all source bytes before every command.
        // Keep partial evidence if an earlier command changed the project.
        let log = logs.join(format!("{}.log", rule.id));
        let preflight = (|| -> Result<PathBuf> {
            require_authorized(&root, &policy)?;
            ensure!(
                inputs(&root, &registry)? == before,
                "stale: project changed during verification"
            );
            executable(&root, rule.verifier.executable())
        })();
        let exe = match preflight {
            Ok(exe) => exe,
            Err(error) => {
                std::fs::write(&log, error.to_string())?;
                results.push(RuleResult {
                    id: rule.id.clone(),
                    status: "inconclusive".into(),
                    assurance: "not_executed".into(),
                    command: vec![],
                    exit_code: None,
                    elapsed_ms: 0,
                    detail: Some(error.to_string()),
                    log: format!("{run_name}/{}.log", rule.id),
                    log_hash: hash(&log)?,
                    artifact: None,
                    artifact_hash: None,
                });
                continue;
            }
        };
        let artifact = logs.join(format!("{}.json", rule.id));
        let (args, cwd, assurance) = match &rule.verifier {
            Verifier::Test { command, .. } => (
                command[1..].to_vec(),
                root.clone(),
                "configured_test_command_exit_status".to_owned(),
            ),
            Verifier::Kani {
                working_directory,
                harness,
                scope,
                ..
            } => {
                ensure!(
                    cfg!(all(target_os = "linux", target_arch = "x86_64")),
                    "Kani adapter requires Linux x86_64"
                );
                for var in [
                    "RUSTFLAGS",
                    "CARGO_ENCODED_RUSTFLAGS",
                    "RUSTC_WRAPPER",
                    "RUSTC_WORKSPACE_WRAPPER",
                    "KANIFLAGS",
                    "CBMC_FLAGS",
                ] {
                    ensure!(
                        std::env::var_os(var).is_none(),
                        "Kani flag override rejected: {var}"
                    );
                }
                (
                    vec![
                        "kani".into(),
                        "--harness".into(),
                        harness.clone(),
                        "--exact".into(),
                        "--output-format".into(),
                        "regular".into(),
                        "-Z".into(),
                        "unstable-options".into(),
                        "--export-json".into(),
                        artifact.to_string_lossy().into(),
                    ],
                    confined(&root, working_directory)?,
                    format!("Kani 0.68.0; declared scope: {scope}"),
                )
            }
        };
        let start = Instant::now();
        let (code, mut detail) = match invoke(&exe, &args, &cwd, &log, rule.verifier.timeout()) {
            Ok((status, detail)) => (status.and_then(|s| s.code()), detail),
            Err(error) => {
                if !log.exists() {
                    std::fs::write(&log, error.to_string())?;
                }
                (None, Some(error.to_string()))
            }
        };
        let status = if detail.is_some() || code.is_none() {
            "inconclusive"
        } else {
            match &rule.verifier {
                Verifier::Test { .. } => {
                    if code == Some(0) {
                        "tests_passed"
                    } else {
                        "failed"
                    }
                }
                Verifier::Kani { harness, .. } => match kani_status(&artifact, harness, code) {
                    Ok(status) => status,
                    Err(error) => {
                        detail = Some(error.to_string());
                        "inconclusive"
                    }
                },
            }
        };
        let artifact_present = artifact.is_file() && matches!(rule.verifier, Verifier::Kani { .. });
        results.push(RuleResult {
            id: rule.id.clone(),
            status: status.into(),
            assurance,
            command: std::iter::once(exe.to_string_lossy().into_owned())
                .chain(args)
                .collect(),
            exit_code: code,
            elapsed_ms: start.elapsed().as_millis(),
            detail,
            log: format!("{run_name}/{}.log", rule.id),
            log_hash: hash(&log)?,
            artifact: artifact_present.then(|| format!("{run_name}/{}.json", rule.id)),
            artifact_hash: if artifact_present {
                Some(hash(&artifact)?)
            } else {
                None
            },
        });
    }
    let unchanged = inputs(&root, &registry).is_ok_and(|current| current == before)
        && candidate_policy(&root).is_ok_and(|current| current == policy)
        && external_policy(&root, policy_path).is_ok_and(|current| current == policy);
    let status = if !unchanged {
        "stale"
    } else if results.iter().any(|r| r.status == "failed") {
        "failed"
    } else if results.iter().all(|r| passed(&r.status)) {
        "passed"
    } else {
        "inconclusive"
    };
    let report = Report { schema_version: 1, status: status.into(), generated_at: chrono::Utc::now().to_rfc3339(),
        revision: crate::evidence::revision(&root), inputs: before, policy, results,
        limitations: vec!["Tests only establish the configured command's exit status; they are not mathematical proofs or proof that every required assertion exists".into(),
            "Formal results apply only to the reviewed harness, its assumptions and declared scope".into(),
            "Only declared inputs are fingerprinted; declare transitive tests, schemas, manifests and lockfiles".into(),
            "External policies require independent review and read-only storage enforced by the operator/CI; this tool is not a sandbox".into(),
            "SHA-256 detects changed bytes, not forged reports or malicious verifier implementations".into()] };
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    Ok(report)
}

pub fn check_report(root: &Path, path: &Path, policy_path: &Path) -> Result<String> {
    ensure!(
        std::fs::metadata(path)?.len() <= 64_000_000,
        "report too large"
    );
    let report: Report = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    ensure!(report.schema_version == 1, "unsupported report schema");
    let registry = match load(root) {
        Ok(registry) => registry,
        Err(_) => return Ok("stale".into()),
    };
    let policy = external_policy(root, policy_path)?;
    if !inputs(root, &registry).is_ok_and(|current| report.inputs == current)
        || report.policy != policy
        || !candidate_policy(root).is_ok_and(|current| current == policy)
    {
        return Ok("stale".into());
    }
    let expected: BTreeSet<_> = registry.rules.iter().map(|r| &r.id).collect();
    let actual: BTreeSet<_> = report.results.iter().map(|r| &r.id).collect();
    if actual != expected || report.results.len() != expected.len() {
        return Ok("inconclusive".into());
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    for result in &report.results {
        let log = match confined(parent, &result.log) {
            Ok(log) => log,
            Err(_) => return Ok("stale".into()),
        };
        if hash(&log)? != result.log_hash {
            return Ok("stale".into());
        }
        let rule = registry.rules.iter().find(|r| r.id == result.id).unwrap();
        match &rule.verifier {
            Verifier::Test { .. } => {
                if result.status != "tests_passed"
                    || result.exit_code != Some(0)
                    || result.detail.is_some()
                {
                    return Ok("inconclusive".into());
                }
            }
            Verifier::Kani { harness, .. } => {
                let artifact = confined(
                    parent,
                    result
                        .artifact
                        .as_deref()
                        .context("missing formal evidence")?,
                )?;
                if Some(hash(&artifact)?) != result.artifact_hash {
                    return Ok("stale".into());
                }
                if result.status != "formally_verified"
                    || kani_status(&artifact, harness, result.exit_code)? != "formally_verified"
                    || result.detail.is_some()
                {
                    return Ok("inconclusive".into());
                }
            }
        }
    }
    Ok(if report.status == "passed" {
        "passed"
    } else {
        "inconclusive"
    }
    .into())
}
