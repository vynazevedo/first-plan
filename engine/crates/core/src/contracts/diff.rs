use super::{openapi::Endpoint, ContractsReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractsDiff {
    pub before_generated_at: String,
    pub after_generated_at: String,
    pub openapi: OpenApiDiff,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenApiDiff {
    pub added: Vec<EndpointChange>,
    pub removed: Vec<EndpointChange>,
    pub modified: Vec<EndpointModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub breaking: usize,
    pub non_breaking: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointChange {
    pub method: String,
    pub path: String,
    pub spec_file: String,
    pub operation_id: Option<String>,
    pub is_breaking: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointModification {
    pub method: String,
    pub path: String,
    pub spec_file: String,
    pub changes: Vec<FieldChange>,
    pub is_breaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub breaking: bool,
}

pub fn diff(before: &ContractsReport, after: &ContractsReport) -> ContractsDiff {
    let openapi = diff_openapi(&before.openapi.endpoints, &after.openapi.endpoints);
    let summary = compute_summary(&openapi);
    ContractsDiff {
        before_generated_at: before.generated_at.clone(),
        after_generated_at: after.generated_at.clone(),
        openapi,
        summary,
    }
}

fn endpoint_key(e: &Endpoint) -> (String, String) {
    (e.method.to_uppercase(), e.path.clone())
}

fn diff_openapi(before: &[Endpoint], after: &[Endpoint]) -> OpenApiDiff {
    let before_map: HashMap<(String, String), &Endpoint> =
        before.iter().map(|e| (endpoint_key(e), e)).collect();
    let after_map: HashMap<(String, String), &Endpoint> =
        after.iter().map(|e| (endpoint_key(e), e)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    for (key, ep_after) in &after_map {
        match before_map.get(key) {
            None => {
                added.push(EndpointChange {
                    method: ep_after.method.clone(),
                    path: ep_after.path.clone(),
                    spec_file: ep_after.spec_file.clone(),
                    operation_id: ep_after.operation_id.clone(),
                    is_breaking: false,
                    reason: "endpoint novo (adição é sempre non-breaking)".to_string(),
                });
            }
            Some(ep_before) => {
                if let Some(m) = diff_endpoint(ep_before, ep_after) {
                    modified.push(m);
                }
            }
        }
    }

    for (key, ep_before) in &before_map {
        if !after_map.contains_key(key) {
            removed.push(EndpointChange {
                method: ep_before.method.clone(),
                path: ep_before.path.clone(),
                spec_file: ep_before.spec_file.clone(),
                operation_id: ep_before.operation_id.clone(),
                is_breaking: true,
                reason: "endpoint removido - consumidores existentes vão receber 404".to_string(),
            });
        }
    }

    added.sort_by(|a, b| (&a.method, &a.path).cmp(&(&b.method, &b.path)));
    removed.sort_by(|a, b| (&a.method, &a.path).cmp(&(&b.method, &b.path)));
    modified.sort_by(|a, b| (&a.method, &a.path).cmp(&(&b.method, &b.path)));

    OpenApiDiff {
        added,
        removed,
        modified,
    }
}

fn diff_endpoint(before: &Endpoint, after: &Endpoint) -> Option<EndpointModification> {
    let mut changes = Vec::new();

    if before.operation_id != after.operation_id {
        changes.push(FieldChange {
            field: "operation_id".to_string(),
            before: before.operation_id.clone(),
            after: after.operation_id.clone(),
            breaking: true,
        });
    }

    if before.summary != after.summary {
        changes.push(FieldChange {
            field: "summary".to_string(),
            before: before.summary.clone(),
            after: after.summary.clone(),
            breaking: false,
        });
    }

    let tags_before = normalize_tags(&before.tags);
    let tags_after = normalize_tags(&after.tags);
    if tags_before != tags_after {
        changes.push(FieldChange {
            field: "tags".to_string(),
            before: Some(tags_before),
            after: Some(tags_after),
            breaking: false,
        });
    }

    if before.spec_file != after.spec_file {
        changes.push(FieldChange {
            field: "spec_file".to_string(),
            before: Some(before.spec_file.clone()),
            after: Some(after.spec_file.clone()),
            breaking: false,
        });
    }

    if changes.is_empty() {
        return None;
    }

    let is_breaking = changes.iter().any(|c| c.breaking);
    Some(EndpointModification {
        method: after.method.clone(),
        path: after.path.clone(),
        spec_file: after.spec_file.clone(),
        changes,
        is_breaking,
    })
}

fn normalize_tags(tags: &[String]) -> String {
    let mut sorted: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
    sorted.sort();
    sorted.join(",")
}

fn compute_summary(openapi: &OpenApiDiff) -> DiffSummary {
    let mut breaking = 0;
    let mut non_breaking = 0;

    for a in &openapi.added {
        if a.is_breaking {
            breaking += 1;
        } else {
            non_breaking += 1;
        }
    }
    for r in &openapi.removed {
        if r.is_breaking {
            breaking += 1;
        } else {
            non_breaking += 1;
        }
    }
    for m in &openapi.modified {
        if m.is_breaking {
            breaking += 1;
        } else {
            non_breaking += 1;
        }
    }

    DiffSummary {
        total_changes: openapi.added.len() + openapi.removed.len() + openapi.modified.len(),
        breaking,
        non_breaking,
    }
}

pub fn render_markdown(diff: &ContractsDiff) -> String {
    let mut s = String::new();
    s.push_str("# Contract diff\n\n");
    s.push_str(&format!("- Before: `{}`\n", diff.before_generated_at));
    s.push_str(&format!("- After: `{}`\n\n", diff.after_generated_at));

    s.push_str("## Summary\n\n");
    s.push_str(&format!(
        "- Total changes: **{}**\n",
        diff.summary.total_changes
    ));
    s.push_str(&format!("- Breaking: **{}**\n", diff.summary.breaking));
    s.push_str(&format!(
        "- Non-breaking: **{}**\n\n",
        diff.summary.non_breaking
    ));

    s.push_str("## OpenAPI\n\n");

    if !diff.openapi.removed.is_empty() {
        s.push_str(&format!("### Removed ({})\n\n", diff.openapi.removed.len()));
        s.push_str("| Method | Path | operation_id | Breaking |\n");
        s.push_str("|--------|------|--------------|----------|\n");
        for r in &diff.openapi.removed {
            s.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                r.method,
                r.path,
                r.operation_id.as_deref().unwrap_or("-"),
                if r.is_breaking { "**yes**" } else { "no" }
            ));
        }
        s.push('\n');
    }

    if !diff.openapi.added.is_empty() {
        s.push_str(&format!("### Added ({})\n\n", diff.openapi.added.len()));
        s.push_str("| Method | Path | operation_id |\n");
        s.push_str("|--------|------|--------------|\n");
        for a in &diff.openapi.added {
            s.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                a.method,
                a.path,
                a.operation_id.as_deref().unwrap_or("-")
            ));
        }
        s.push('\n');
    }

    if !diff.openapi.modified.is_empty() {
        s.push_str(&format!(
            "### Modified ({})\n\n",
            diff.openapi.modified.len()
        ));
        for m in &diff.openapi.modified {
            let marker = if m.is_breaking { "BREAKING" } else { "safe" };
            s.push_str(&format!(
                "#### `{}` `{}` ({})\n\n",
                m.method, m.path, marker
            ));
            s.push_str("| Field | Before | After | Breaking |\n");
            s.push_str("|-------|--------|-------|----------|\n");
            for c in &m.changes {
                s.push_str(&format!(
                    "| `{}` | {} | {} | {} |\n",
                    c.field,
                    c.before.as_deref().unwrap_or("-"),
                    c.after.as_deref().unwrap_or("-"),
                    if c.breaking { "**yes**" } else { "no" }
                ));
            }
            s.push('\n');
        }
    }

    if diff.summary.total_changes == 0 {
        s.push_str("_Nenhuma mudança detectada entre os snapshots._\n");
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{
        crossref::CrossrefReport, graphql::GraphqlReport, openapi::OpenApiReport,
        protobuf::ProtobufReport,
    };

    fn empty_report(when: &str) -> ContractsReport {
        ContractsReport {
            generated_at: when.to_string(),
            elapsed_ms: 0,
            root: "/tmp".to_string(),
            openapi: OpenApiReport::default(),
            protobuf: ProtobufReport::default(),
            graphql: GraphqlReport::default(),
            crossref: CrossrefReport::default(),
        }
    }

    fn ep(method: &str, path: &str, op_id: Option<&str>) -> Endpoint {
        Endpoint {
            spec_file: "openapi.yaml".to_string(),
            path: path.to_string(),
            method: method.to_string(),
            operation_id: op_id.map(String::from),
            summary: None,
            tags: vec![],
        }
    }

    #[test]
    fn diff_detects_added_endpoint_as_non_breaking() {
        let before = empty_report("2026-01-01T00:00:00Z");
        let mut after = empty_report("2026-01-02T00:00:00Z");
        after
            .openapi
            .endpoints
            .push(ep("GET", "/users", Some("listUsers")));

        let d = diff(&before, &after);
        assert_eq!(d.openapi.added.len(), 1);
        assert_eq!(d.openapi.removed.len(), 0);
        assert!(!d.openapi.added[0].is_breaking);
        assert_eq!(d.summary.breaking, 0);
        assert_eq!(d.summary.non_breaking, 1);
    }

    #[test]
    fn diff_detects_removed_endpoint_as_breaking() {
        let mut before = empty_report("2026-01-01T00:00:00Z");
        let after = empty_report("2026-01-02T00:00:00Z");
        before
            .openapi
            .endpoints
            .push(ep("DELETE", "/legacy", Some("deleteLegacy")));

        let d = diff(&before, &after);
        assert_eq!(d.openapi.removed.len(), 1);
        assert!(d.openapi.removed[0].is_breaking);
        assert!(d.openapi.removed[0].reason.contains("404"));
        assert_eq!(d.summary.breaking, 1);
    }

    #[test]
    fn diff_operation_id_change_is_breaking() {
        let mut before = empty_report("2026-01-01T00:00:00Z");
        let mut after = empty_report("2026-01-02T00:00:00Z");
        before.openapi.endpoints.push(ep("GET", "/x", Some("getX")));
        after
            .openapi
            .endpoints
            .push(ep("GET", "/x", Some("fetchX")));

        let d = diff(&before, &after);
        assert_eq!(d.openapi.modified.len(), 1);
        assert!(d.openapi.modified[0].is_breaking);
        assert_eq!(d.openapi.modified[0].changes[0].field, "operation_id");
    }

    #[test]
    fn diff_summary_only_change_is_not_breaking() {
        let mut before = empty_report("2026-01-01T00:00:00Z");
        let mut after = empty_report("2026-01-02T00:00:00Z");
        let mut e1 = ep("GET", "/x", Some("getX"));
        e1.summary = Some("Old summary".to_string());
        let mut e2 = ep("GET", "/x", Some("getX"));
        e2.summary = Some("New summary".to_string());
        before.openapi.endpoints.push(e1);
        after.openapi.endpoints.push(e2);

        let d = diff(&before, &after);
        assert_eq!(d.openapi.modified.len(), 1);
        assert!(!d.openapi.modified[0].is_breaking);
        assert_eq!(d.summary.breaking, 0);
        assert_eq!(d.summary.non_breaking, 1);
    }

    #[test]
    fn diff_identical_reports_produce_zero_changes() {
        let mut before = empty_report("2026-01-01T00:00:00Z");
        let mut after = empty_report("2026-01-02T00:00:00Z");
        before.openapi.endpoints.push(ep("GET", "/x", Some("getX")));
        after.openapi.endpoints.push(ep("GET", "/x", Some("getX")));

        let d = diff(&before, &after);
        assert_eq!(d.summary.total_changes, 0);
        assert_eq!(d.summary.breaking, 0);
    }

    #[test]
    fn diff_method_change_shows_as_add_plus_remove() {
        let mut before = empty_report("2026-01-01T00:00:00Z");
        let mut after = empty_report("2026-01-02T00:00:00Z");
        before.openapi.endpoints.push(ep("POST", "/users", None));
        after.openapi.endpoints.push(ep("PUT", "/users", None));

        let d = diff(&before, &after);
        assert_eq!(d.openapi.added.len(), 1);
        assert_eq!(d.openapi.removed.len(), 1);
        assert_eq!(d.openapi.added[0].method, "PUT");
        assert_eq!(d.openapi.removed[0].method, "POST");
        assert!(d.openapi.removed[0].is_breaking);
    }
}
