//! Cross-repository references to API contracts. Matches are candidates, not runtime proof.
use crate::{contracts::openapi, evidence, multirepo};
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct Impact {
    pub schema_version: u32,
    pub references: Vec<Reference>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Reference {
    pub producer: String,
    pub consumer: String,
    pub contract_file: String,
    pub method: String,
    pub endpoint: String,
    pub matched_identifier: String,
    pub status: String,
    pub evidence: evidence::Evidence,
}

pub fn analyze(root: &Path) -> Result<Impact> {
    let cfg = multirepo::config::load(root)?;
    let mut repos = vec![("self".to_owned(), root.to_path_buf())];
    repos.extend(
        cfg.repos
            .iter()
            .map(|r| (r.name.clone(), multirepo::config::resolved_path(root, r))),
    );
    let mut report = Impact { schema_version: 1, references: vec![], warnings: vec![
        "Static identifier/path matches are candidate consumers, not proven HTTP calls. Dynamic URLs, generated clients and external repos may be missed".into()
    ] };
    for (producer, path) in &repos {
        let contracts = openapi::detect(path);
        report.warnings.extend(contracts.warnings);
        for (consumer, consumer_path) in &repos {
            if producer == consumer {
                continue;
            }
            let files = match evidence::files(consumer_path) {
                Ok(files) => files,
                Err(e) => {
                    report.warnings.push(format!("{}: {}", consumer, e));
                    continue;
                }
            };
            for file in files {
                if crate::symbols::language_from_path(&file).is_none() {
                    continue;
                }
                let Some(text) = evidence::read(consumer_path, &file) else {
                    continue;
                };
                for endpoint in &contracts.endpoints {
                    for (line, value) in text.lines().enumerate() {
                        let identifier = endpoint
                            .operation_id
                            .as_deref()
                            .filter(|s| s.len() >= 3 && value.contains(s))
                            .or_else(|| {
                                (endpoint.path.len() > 1 && value.contains(&endpoint.path))
                                    .then_some(endpoint.path.as_str())
                            });
                        let Some(identifier) = identifier else {
                            continue;
                        };
                        report.references.push(Reference {
                            producer: producer.clone(),
                            consumer: consumer.clone(),
                            contract_file: endpoint.spec_file.clone(),
                            method: endpoint.method.clone(),
                            endpoint: endpoint.path.clone(),
                            matched_identifier: identifier.into(),
                            status: "candidate".into(),
                            evidence: evidence::source(
                                &file,
                                &text,
                                line + 1,
                                "observed_reference",
                            ),
                        });
                    }
                }
            }
        }
    }
    report.references.sort_by(|a, b| {
        (
            &a.producer,
            &a.consumer,
            &a.endpoint,
            &a.method,
            &a.evidence.path,
            a.evidence.line,
        )
            .cmp(&(
                &b.producer,
                &b.consumer,
                &b.endpoint,
                &b.method,
                &b.evidence.path,
                b.evidence.line,
            ))
    });
    report.warnings.sort();
    report.warnings.dedup();
    Ok(report)
}
