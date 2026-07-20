//! Contracts Layer (v0.9.0).
//!
//! Detecta contratos formais entre componentes: OpenAPI specs, Protobuf services,
//! GraphQL schemas. Faz cross-reference com o codigo para classificar cada endpoint
//! ou RPC como IMPLEMENTED, PHANTOM ou DRIFTED.
//!
//! Foundation para v0.10 (Evolution que valida migracoes de spec) e v0.11 (Runtime
//! link que precisa saber quais endpoints estao vivos em prod).

pub mod crossref;
pub mod graphql;
pub mod openapi;
pub mod protobuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractsReport {
    pub generated_at: String,
    pub elapsed_ms: u64,
    pub root: String,
    pub openapi: openapi::OpenApiReport,
    pub protobuf: protobuf::ProtobufReport,
    pub graphql: graphql::GraphqlReport,
    pub crossref: crossref::CrossrefReport,
}

pub fn analyze(root: &std::path::Path) -> ContractsReport {
    let start = std::time::Instant::now();
    let openapi = openapi::detect(root);
    let protobuf = protobuf::detect(root);
    let graphql = graphql::detect(root);
    let crossref = crossref::analyze(root, &openapi, &protobuf, &graphql);
    ContractsReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        root: root.to_string_lossy().into_owned(),
        openapi,
        protobuf,
        graphql,
        crossref,
    }
}
