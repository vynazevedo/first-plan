//! GraphQL SDL detection and parsing.
//!
//! Suporta:
//! - *.graphql, *.gql em qualquer diretorio
//! - schema.graphql em locais comuns
//!
//! Extrai Query/Mutation/Subscription fields + Object/Interface/Enum types.
//! Output: cada campo top-level como "operation" que crossref busca no codigo.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphqlReport {
    pub schemas_found: Vec<SchemaFile>,
    pub operations: Vec<GqlOperation>,
    pub types: Vec<GqlType>,
    pub total_operations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFile {
    pub path: String,
    pub operation_count: usize,
    pub type_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GqlOperation {
    pub schema_file: String,
    pub kind: GqlOperationKind,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GqlOperationKind {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GqlType {
    pub schema_file: String,
    pub name: String,
    pub kind: GqlTypeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GqlTypeKind {
    Object,
    Interface,
    Union,
    Enum,
    Input,
    Scalar,
}

const EXCLUDED_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "vendor",
    ".git",
    "dist",
    "build",
    ".first-plan",
];

pub fn detect(root: &Path) -> GraphqlReport {
    let mut report = GraphqlReport::default();
    for path in find_schema_files(root) {
        parse_schema(&path, root, &mut report);
    }
    report.total_operations = report.operations.len();
    report
}

fn find_schema_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(10)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|d| name == *d)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext.eq_ignore_ascii_case("graphql") || ext.eq_ignore_ascii_case("gql") {
            out.push(entry.path().to_path_buf());
        }
    }
    out
}

fn parse_schema(path: &Path, root: &Path, report: &mut GraphqlReport) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    let doc = match graphql_parser::parse_schema::<String>(&content) {
        Ok(d) => d,
        Err(_) => return,
    };

    let mut op_count = 0usize;
    let mut type_count = 0usize;

    use graphql_parser::schema::{Definition, TypeDefinition};

    for def in doc.definitions {
        if let Definition::TypeDefinition(td) = def {
            match td {
                TypeDefinition::Object(obj) => {
                    let is_root =
                        matches!(obj.name.as_str(), "Query" | "Mutation" | "Subscription");
                    if is_root {
                        let kind = match obj.name.as_str() {
                            "Query" => GqlOperationKind::Query,
                            "Mutation" => GqlOperationKind::Mutation,
                            "Subscription" => GqlOperationKind::Subscription,
                            _ => unreachable!(),
                        };
                        for field in obj.fields {
                            report.operations.push(GqlOperation {
                                schema_file: rel.clone(),
                                kind,
                                name: field.name.clone(),
                                description: field.description.clone(),
                            });
                            op_count += 1;
                        }
                    } else {
                        report.types.push(GqlType {
                            schema_file: rel.clone(),
                            name: obj.name.clone(),
                            kind: GqlTypeKind::Object,
                        });
                        type_count += 1;
                    }
                }
                TypeDefinition::Interface(iface) => {
                    report.types.push(GqlType {
                        schema_file: rel.clone(),
                        name: iface.name.clone(),
                        kind: GqlTypeKind::Interface,
                    });
                    type_count += 1;
                }
                TypeDefinition::Union(u) => {
                    report.types.push(GqlType {
                        schema_file: rel.clone(),
                        name: u.name.clone(),
                        kind: GqlTypeKind::Union,
                    });
                    type_count += 1;
                }
                TypeDefinition::Enum(e) => {
                    report.types.push(GqlType {
                        schema_file: rel.clone(),
                        name: e.name.clone(),
                        kind: GqlTypeKind::Enum,
                    });
                    type_count += 1;
                }
                TypeDefinition::InputObject(io) => {
                    report.types.push(GqlType {
                        schema_file: rel.clone(),
                        name: io.name.clone(),
                        kind: GqlTypeKind::Input,
                    });
                    type_count += 1;
                }
                TypeDefinition::Scalar(s) => {
                    report.types.push(GqlType {
                        schema_file: rel.clone(),
                        name: s.name.clone(),
                        kind: GqlTypeKind::Scalar,
                    });
                    type_count += 1;
                }
            }
        }
    }

    report.schemas_found.push(SchemaFile {
        path: rel,
        operation_count: op_count,
        type_count,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_basic_schema() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("schema.graphql"),
            r#"
type Query {
  user(id: ID!): User
  users: [User!]!
}

type Mutation {
  createUser(input: CreateUserInput!): User
}

type User {
  id: ID!
  name: String!
}

input CreateUserInput {
  name: String!
}
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.schemas_found.len(), 1);
        assert!(report.operations.iter().any(|o| o.name == "user"));
        assert!(report.operations.iter().any(|o| o.name == "users"));
        assert!(report.operations.iter().any(|o| o.name == "createUser"));
        assert!(report.types.iter().any(|t| t.name == "User"));
        assert!(report.types.iter().any(|t| t.name == "CreateUserInput"));
    }

    #[test]
    fn ignores_excluded_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("node_modules/foo")).unwrap();
        fs::write(
            tmp.path().join("node_modules/foo/x.graphql"),
            "type Query { x: String }",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert_eq!(report.schemas_found.len(), 0);
    }

    #[test]
    fn empty_when_no_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.schemas_found.len(), 0);
    }
}
