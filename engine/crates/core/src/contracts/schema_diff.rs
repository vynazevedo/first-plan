//! Conservative structural compatibility checks. Unknown changes require review.
use super::diff::FieldChange;
use serde_json::Value;
use std::collections::BTreeSet;

pub fn compare(before: &Value, after: &Value, path: &str, changes: &mut Vec<FieldChange>) {
    if before == after {
        return;
    }
    if let (Some(old), Some(new)) = (before.as_object(), after.as_object()) {
        let keys: BTreeSet<_> = old.keys().chain(new.keys()).collect();
        for key in keys {
            let a = old.get(key).unwrap_or(&Value::Null);
            let b = new.get(key).unwrap_or(&Value::Null);
            let field = format!("{}/{}", path, key);
            if a == b {
                continue;
            }
            if matches!(
                key.as_str(),
                "description"
                    | "summary"
                    | "title"
                    | "example"
                    | "examples"
                    | "deprecated"
                    | "externalDocs"
            ) {
                record(a, b, &field, false, changes);
            } else if key == "required" || key == "enum" {
                let request = !path.contains("/responses/");
                let breaking = if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
                    if (key == "required") == request {
                        b.iter().any(|v| !a.contains(v))
                    } else {
                        a.iter().any(|v| !b.contains(v))
                    }
                } else if key == "required" && (a.is_boolean() || b.is_boolean()) {
                    b.as_bool().unwrap_or(false) && !a.as_bool().unwrap_or(false)
                } else {
                    true
                };
                record(a, b, &field, breaking, changes);
            } else if a.is_null() || b.is_null() {
                let additive_property = a.is_null() && path.ends_with("/properties");
                let optional_parameter = a.is_null()
                    && path.ends_with("/parameters")
                    && b.get("required").and_then(Value::as_bool) != Some(true);
                let optional_body = a.is_null()
                    && key == "requestBody"
                    && b.get("required").and_then(Value::as_bool) != Some(true);
                let response_addition = a.is_null() && path.ends_with("/responses");
                record(
                    a,
                    b,
                    &field,
                    !(additive_property
                        || optional_parameter
                        || optional_body
                        || response_addition),
                    changes,
                );
            } else {
                compare(a, b, &field, changes);
            }
        }
    } else {
        record(before, after, path, true, changes);
    }
}

fn record(
    before: &Value,
    after: &Value,
    path: &str,
    breaking: bool,
    changes: &mut Vec<FieldChange>,
) {
    changes.push(FieldChange {
        field: path.into(),
        before: Some(before.to_string()),
        after: Some(after.to_string()),
        breaking,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn detects_required_input_and_response_type_changes() {
        let a = json!({"parameters": {}, "responses": {"200": {"schema": {"type": "string"}}}});
        let b = json!({"parameters": {"query:id": {"required": true}}, "responses": {"200": {"schema": {"type": "integer"}}}});
        let mut changes = vec![];
        compare(&a, &b, "contract", &mut changes);
        assert_eq!(changes.iter().filter(|c| c.breaking).count(), 2);
    }
    #[test]
    fn optional_parameter_and_description_are_safe() {
        let mut changes = vec![];
        compare(
            &json!({"parameters": {}}),
            &json!({"parameters": {"query:page": {"required": false}}}),
            "contract",
            &mut changes,
        );
        assert!(changes.iter().all(|c| !c.breaking));
    }
    #[test]
    fn enum_direction_depends_on_request_or_response() {
        for (path, expected) in [
            ("contract/requestBody/schema", false),
            ("contract/responses/200/schema", true),
        ] {
            let mut changes = vec![];
            compare(
                &json!({"enum": ["a"]}),
                &json!({"enum": ["a", "b"]}),
                path,
                &mut changes,
            );
            assert_eq!(changes[0].breaking, expected);
        }
    }
}
