//! Reviewed project requirements and evidence-bound verification policies.
mod execution;
pub use execution::{check_report, run, Report, RuleResult};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub const REGISTRY: &str = ".first-plan/rules.yaml";
const MAX_FILES: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: u32,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub requirement: String,
    pub owner: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub inputs: Vec<String>,
    pub verification_files: Vec<String>,
    pub verifier: Verifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Verifier {
    Test {
        command: Vec<String>,
        timeout_seconds: u64,
    },
    Kani {
        executable: String,
        working_directory: String,
        harness: String,
        scope: String,
        timeout_seconds: u64,
    },
}

impl Verifier {
    pub fn executable(&self) -> &str {
        match self {
            Self::Test { command, .. } => &command[0],
            Self::Kani { executable, .. } => executable,
        }
    }
    pub fn timeout(&self) -> u64 {
        match self {
            Self::Test {
                timeout_seconds, ..
            }
            | Self::Kani {
                timeout_seconds, ..
            } => *timeout_seconds,
        }
    }
}

pub fn relative(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty() && !path.contains('\\') && !path.contains(':') && !path.contains('\0'),
        "invalid relative path: {path}"
    );
    ensure!(
        Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_))),
        "path must be relative without dot/parent components: {path}"
    );
    ensure!(
        !path.split('/').any(|p| p == ".git"),
        "Git internals cannot be verification inputs"
    );
    Ok(())
}

pub fn confined(root: &Path, path: &str) -> Result<PathBuf> {
    relative(path)?;
    let root = root.canonicalize()?;
    let full = root.join(path);
    for ancestor in full.ancestors().take_while(|p| *p != root) {
        ensure!(
            !std::fs::symlink_metadata(ancestor)?
                .file_type()
                .is_symlink(),
            "symlink input rejected: {path}"
        );
    }
    let full = full.canonicalize()?;
    ensure!(full.starts_with(root), "input escapes root: {path}");
    Ok(full)
}

pub fn load(root: &Path) -> Result<Registry> {
    let path = confined(root, REGISTRY)?;
    ensure!(
        std::fs::metadata(&path)?.len() <= 256_000,
        "rules registry exceeds 256KB"
    );
    let registry: Registry = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
    ensure!(registry.schema_version == 1, "unsupported rules schema");
    ensure!(
        !registry.rules.is_empty() && registry.rules.len() <= 100,
        "registry needs 1..100 rules"
    );
    let mut ids = BTreeSet::new();
    for rule in &registry.rules {
        ensure!(
            !rule.id.is_empty()
                && rule.id.len() <= 80
                && rule
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "invalid rule id"
        );
        ensure!(ids.insert(&rule.id), "duplicate rule id: {}", rule.id);
        ensure!(
            !rule.requirement.trim().is_empty()
                && rule.requirement.len() <= 4000
                && !rule.owner.trim().is_empty(),
            "rule needs a bounded requirement and owner: {}",
            rule.id
        );
        ensure!(
            !rule.inputs.is_empty() && !rule.verification_files.is_empty(),
            "rule needs inputs and verification_files: {}",
            rule.id
        );
        for path in rule.inputs.iter().chain(&rule.verification_files) {
            relative(path)?;
        }
        ensure!(
            (1..=600).contains(&rule.verifier.timeout()),
            "timeout must be 1..600 seconds"
        );
        match &rule.verifier {
            Verifier::Test { command, .. } => {
                ensure!(
                    !command.is_empty()
                        && command.len() <= 100
                        && command.iter().all(|s| !s.contains('\0') && s.len() <= 8192)
                        && !command[0].is_empty(),
                    "invalid test command"
                );
            }
            Verifier::Kani {
                working_directory,
                harness,
                scope,
                executable,
                ..
            } => {
                relative(working_directory)?;
                ensure!(
                    !executable.is_empty()
                        && !scope.trim().is_empty()
                        && !harness.is_empty()
                        && harness
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b':'),
                    "Kani needs executable, exact harness and explicit scope"
                );
            }
        }
    }
    Ok(registry)
}

pub fn hash(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", digest.finalize()))
}

pub fn snapshot(
    root: &Path,
    paths: impl IntoIterator<Item = String>,
) -> Result<BTreeMap<String, String>> {
    let canonical_root = root.canonicalize()?;
    let mut result = BTreeMap::new();
    for path in paths {
        let full =
            confined(root, &path).with_context(|| format!("missing or invalid input {path}"))?;
        let entries: Vec<PathBuf> = if full.is_dir() {
            let mut entries = Vec::new();
            for entry in walkdir::WalkDir::new(&full)
                .follow_links(false)
                .max_depth(64)
            {
                let entry = entry?;
                ensure!(
                    entry.path().canonicalize()?.starts_with(&canonical_root),
                    "declared input escapes project root"
                );
                ensure!(
                    entry.depth() < 64 || !entry.file_type().is_dir(),
                    "declared input directory exceeds depth limit"
                );
                ensure!(
                    !entry.file_type().is_symlink(),
                    "symlink in declared input: {}",
                    entry.path().display()
                );
                if entry.file_type().is_file() {
                    entries.push(entry.path().to_owned());
                }
                ensure!(entries.len() <= MAX_FILES, "too many declared input files");
            }
            ensure!(!entries.is_empty(), "empty input directory: {path}");
            entries
        } else {
            vec![full]
        };
        for file in entries {
            ensure!(
                std::fs::metadata(&file)?.is_file(),
                "input is not a regular file"
            );
            let key = file
                .strip_prefix(root.canonicalize()?)?
                .to_string_lossy()
                .replace('\\', "/");
            let checked = confined(root, &key)?;
            result.insert(key, hash(&checked)?);
            ensure!(result.len() <= MAX_FILES, "too many verification files");
        }
    }
    Ok(result)
}

pub fn inputs(root: &Path, registry: &Registry) -> Result<BTreeMap<String, String>> {
    snapshot(
        root,
        std::iter::once(REGISTRY.to_owned()).chain(
            registry
                .rules
                .iter()
                .flat_map(|r| r.inputs.iter().chain(&r.verification_files).cloned()),
        ),
    )
}

pub fn executable(root: &Path, name: &str) -> Result<PathBuf> {
    let path = if Path::new(name).is_absolute() {
        PathBuf::from(name)
    } else if name.contains('/') || name.contains('\\') {
        confined(root, name)?
    } else {
        which::which(name).with_context(|| format!("verifier not installed: {name}"))?
    };
    Ok(path.canonicalize()?)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema_version: u32,
    pub registry_hash: String,
    pub rule_ids: Vec<String>,
    pub rules: Vec<Rule>,
    pub verification_files: BTreeMap<String, String>,
    pub executables: BTreeMap<String, String>,
}

pub fn candidate_policy(root: &Path) -> Result<Policy> {
    let registry = load(root)?;
    // Validate declared source availability too, without freezing implementation changes.
    inputs(root, &registry)?;
    let verification_files = snapshot(
        root,
        registry
            .rules
            .iter()
            .flat_map(|r| r.verification_files.clone()),
    )?;
    let mut executables = BTreeMap::new();
    for rule in &registry.rules {
        executables.insert(
            rule.id.clone(),
            hash(&executable(root, rule.verifier.executable())?)?,
        );
    }
    let policy = Policy {
        schema_version: 1,
        registry_hash: hash(&root.join(REGISTRY))?,
        rule_ids: registry.rules.iter().map(|r| r.id.clone()).collect(),
        rules: registry.rules.clone(),
        verification_files,
        executables,
    };
    ensure!(
        serde_json::to_vec_pretty(&policy)?.len() <= 2_000_000,
        "policy snapshot exceeds 2MB; narrow verification_files"
    );
    Ok(policy)
}

pub fn external_policy(root: &Path, path: &Path) -> Result<Policy> {
    ensure!(
        !path.canonicalize()?.starts_with(root.canonicalize()?),
        "execution policy must be supplied outside the project root"
    );
    ensure!(
        std::fs::metadata(path)?.len() <= 2_000_000,
        "policy too large"
    );
    let policy: Policy = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    ensure!(
        policy.schema_version == 1 && !policy.rule_ids.is_empty(),
        "invalid policy"
    );
    Ok(policy)
}

#[derive(Debug, Serialize)]
pub struct RuleChange {
    pub id: String,
    pub before: Option<Rule>,
    pub after: Option<Rule>,
}

#[derive(Debug, Serialize)]
pub struct Review {
    pub status: String,
    pub added_rules: Vec<String>,
    pub removed_rules: Vec<String>,
    pub registry_changed: bool,
    pub changed_rules: Vec<RuleChange>,
    pub changed_verification_files: Vec<String>,
    pub changed_executables: Vec<String>,
}

pub fn review(root: &Path, policy: &Policy) -> Result<Review> {
    let current = candidate_policy(root)?;
    let changed = |a: &BTreeMap<String, String>, b: &BTreeMap<String, String>| -> Vec<String> {
        a.keys()
            .chain(b.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|k| a.get(*k) != b.get(*k))
            .cloned()
            .collect()
    };
    Ok(Review {
        status: if &current == policy {
            "authorized"
        } else {
            "review_required"
        }
        .into(),
        added_rules: current
            .rule_ids
            .iter()
            .filter(|id| !policy.rule_ids.contains(id))
            .cloned()
            .collect(),
        removed_rules: policy
            .rule_ids
            .iter()
            .filter(|id| !current.rule_ids.contains(id))
            .cloned()
            .collect(),
        registry_changed: current.registry_hash != policy.registry_hash,
        changed_rules: current
            .rule_ids
            .iter()
            .chain(&policy.rule_ids)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|id| {
                let before = policy.rules.iter().find(|r| &r.id == id).cloned();
                let after = current.rules.iter().find(|r| &r.id == id).cloned();
                (before != after).then(|| RuleChange {
                    id: id.clone(),
                    before,
                    after,
                })
            })
            .collect(),
        changed_verification_files: changed(
            &current.verification_files,
            &policy.verification_files,
        ),
        changed_executables: changed(&current.executables, &policy.executables),
    })
}

pub fn require_authorized(root: &Path, policy: &Policy) -> Result<()> {
    let review = review(root, policy)?;
    if review.status != "authorized" {
        bail!("review_required: {}", serde_json::to_string(&review)?);
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub id: String,
    pub owner: String,
    pub requirement: String,
    pub verification_files: Vec<String>,
    pub match_reason: String,
    pub registry_hash: String,
    pub evidence_status: String,
}

pub fn applicable(root: &Path, query: &str, paths: &[String]) -> Result<Vec<Obligation>> {
    for path in paths {
        relative(path)?;
    }
    if !root.join(REGISTRY).try_exists()? {
        return Ok(Vec::new());
    }
    let registry = load(root)?;
    let registry_hash = hash(&confined(root, REGISTRY)?)?;
    let tokens = crate::tokenize::tokenize(query);
    Ok(registry
        .rules
        .into_iter()
        .filter_map(|r| {
            let path_match = paths.iter().any(|p| {
                r.inputs
                    .iter()
                    .chain(&r.verification_files)
                    .any(|input| p == input || p.starts_with(&format!("{input}/")))
            });
            let text = format!(
                "{} {} {} {}",
                r.id,
                r.requirement,
                r.keywords.join(" "),
                r.inputs.join(" ")
            );
            let words = crate::tokenize::tokenize(&text);
            let lexical = tokens.iter().any(|t| words.contains(t));
            (path_match || lexical).then(|| Obligation {
                id: r.id,
                owner: r.owner,
                requirement: r.requirement,
                verification_files: r.verification_files,
                registry_hash: registry_hash.clone(),
                evidence_status: "required_not_verified".into(),
                match_reason: if path_match {
                    "declared_path"
                } else {
                    "lexical_candidate"
                }
                .into(),
            })
        })
        .collect())
}

#[cfg(test)]
mod tests;
