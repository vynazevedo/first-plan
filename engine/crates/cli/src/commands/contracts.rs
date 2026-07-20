use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_warning,
    OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::contracts::{
    analyze,
    crossref::{CrossrefStatus, SchemaSource},
    ContractsReport,
};
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    #[arg(long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let mode = output_mode(args.json);
    let report = analyze(&args.root);

    let out_dir = args
        .output
        .clone()
        .unwrap_or_else(|| args.root.join(".first-plan").join("12-contracts"));

    std::fs::create_dir_all(&out_dir)?;
    std::fs::write(out_dir.join("00-openapi.md"), render_openapi_md(&report))?;
    std::fs::write(out_dir.join("01-protobuf.md"), render_protobuf_md(&report))?;
    std::fs::write(out_dir.join("02-graphql.md"), render_graphql_md(&report))?;
    std::fs::write(out_dir.join("03-drift.md"), render_drift_md(&report))?;
    std::fs::write(
        out_dir.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;

    if mode == OutputMode::Json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    render_pretty(&report, &out_dir);
    Ok(())
}

fn render_pretty(report: &ContractsReport, out_dir: &std::path::Path) {
    print_header(&format!("Contracts Layer ({}ms)", report.elapsed_ms));

    print_section("OpenAPI");
    if report.openapi.specs_found.is_empty() {
        print_warning("Nenhuma spec OpenAPI encontrada");
    } else {
        print_kv_bold(
            "Specs",
            &report.openapi.specs_found.len().to_string(),
            Color::Green,
        );
        print_kv(
            "Endpoints",
            &report.openapi.total_endpoints.to_string(),
            Color::White,
        );
        for spec in &report.openapi.specs_found {
            println!(
                "  {} {} {}",
                spec.path.as_str().with(Color::Cyan),
                spec.title.as_deref().unwrap_or("").bold(),
                format!("({} endpoints)", spec.endpoint_count).dim()
            );
        }
    }

    print_section("Protobuf");
    if report.protobuf.files_found.is_empty() {
        print_warning("Nenhum arquivo .proto encontrado");
    } else {
        print_kv_bold(
            "Files",
            &report.protobuf.files_found.len().to_string(),
            Color::Green,
        );
        print_kv(
            "Services",
            &report.protobuf.total_services.to_string(),
            Color::White,
        );
        print_kv(
            "RPCs",
            &report.protobuf.total_rpcs.to_string(),
            Color::White,
        );
        for svc in report.protobuf.services.iter().take(10) {
            println!(
                "  {} {} {}",
                svc.file.as_str().with(Color::Cyan),
                svc.name.as_str().bold(),
                format!("({} RPCs)", svc.rpcs.len()).dim()
            );
        }
    }

    print_section("GraphQL");
    if report.graphql.schemas_found.is_empty() {
        print_warning("Nenhum schema GraphQL encontrado");
    } else {
        print_kv_bold(
            "Schemas",
            &report.graphql.schemas_found.len().to_string(),
            Color::Green,
        );
        print_kv(
            "Operations",
            &report.graphql.total_operations.to_string(),
            Color::White,
        );
        print_kv(
            "Types",
            &report.graphql.types.len().to_string(),
            Color::White,
        );
    }

    print_section("Cross-reference (spec vs code)");
    let s = &report.crossref.summary;
    if s.total == 0 {
        print_warning("Nenhum contrato para cross-reference");
    } else {
        print_kv("Total entities", &s.total.to_string(), Color::White);
        print_kv_bold(
            "Implemented",
            &s.implemented.to_string(),
            if s.implemented > 0 {
                Color::Green
            } else {
                Color::DarkGrey
            },
        );
        print_kv_bold(
            "Candidates",
            &s.candidates.to_string(),
            if s.candidates > 0 {
                Color::Yellow
            } else {
                Color::DarkGrey
            },
        );
        print_kv_bold(
            "Phantoms",
            &s.phantoms.to_string(),
            if s.phantoms > 0 {
                Color::Red
            } else {
                Color::DarkGrey
            },
        );
        let phantoms: Vec<_> = report
            .crossref
            .items
            .iter()
            .filter(|i| matches!(i.status, CrossrefStatus::Phantom))
            .take(10)
            .collect();
        if !phantoms.is_empty() {
            println!();
            println!("  {}", "Top phantom candidates:".dim());
            for p in phantoms {
                println!(
                    "    {} {} {}",
                    "PHANTOM".with(Color::Red),
                    p.identifier.as_str().bold().with(Color::White),
                    format!("(from {})", source_label(&p.source)).dim()
                );
            }
        }
    }

    println!();
    print_kv_bold("Saved to", &out_dir.to_string_lossy(), Color::Green);
    flush();
}

fn source_label(src: &SchemaSource) -> String {
    match src {
        SchemaSource::OpenApi { file } => format!("openapi: {}", file),
        SchemaSource::Protobuf { file, service } => format!("proto: {}::{}", file, service),
        SchemaSource::Graphql { file, kind } => format!("graphql {}: {}", kind, file),
    }
}

fn render_openapi_md(report: &ContractsReport) -> String {
    let mut s = String::new();
    s.push_str("# OpenAPI Contracts\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine contracts` at {}\n\n",
        report.generated_at
    ));

    if report.openapi.specs_found.is_empty() {
        s.push_str("Nenhuma spec OpenAPI detectada. Locais checados:\n\n");
        s.push_str("- root: `openapi.yaml`, `openapi.json`, `swagger.yaml`, `api-docs.yaml`\n");
        s.push_str("- diretorios: `docs/`, `api/`, `spec/`, `openapi/`, `docs/api/`\n");
        return s;
    }

    s.push_str("## Specs\n\n");
    for spec in &report.openapi.specs_found {
        s.push_str(&format!(
            "- **{}** ({} endpoints){}\n",
            spec.path,
            spec.endpoint_count,
            spec.title
                .as_ref()
                .map(|t| format!(" - {}", t))
                .unwrap_or_default()
        ));
    }
    s.push('\n');

    s.push_str("## Endpoints\n\n");
    s.push_str("| Method | Path | OperationId | Status |\n");
    s.push_str("|--------|------|-------------|--------|\n");
    for endpoint in &report.openapi.endpoints {
        let status = report
            .crossref
            .items
            .iter()
            .find(|i| {
                matches!(&i.source, SchemaSource::OpenApi { file } if file == &endpoint.spec_file)
                    && i.path.as_deref() == Some(&endpoint.path)
                    && i.method.as_deref() == Some(&endpoint.method)
            })
            .map(|i| format!("{:?}", i.status).to_uppercase())
            .unwrap_or_else(|| "?".to_string());
        s.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            endpoint.method,
            endpoint.path,
            endpoint.operation_id.as_deref().unwrap_or("-"),
            status
        ));
    }
    s.push('\n');

    s.push_str("---\n\n");
    s.push_str("**Como usar**: PHANTOM = endpoint declarado na spec mas sem implementacao no codigo. Antes de assumir que existe, verificar. CANDIDATE = evidencia fraca, requer confirmacao. IMPLEMENTED = referencias multiplas confirmam existencia.\n");
    s
}

fn render_protobuf_md(report: &ContractsReport) -> String {
    let mut s = String::new();
    s.push_str("# Protobuf Contracts\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine contracts` at {}\n\n",
        report.generated_at
    ));

    if report.protobuf.files_found.is_empty() {
        s.push_str("Nenhum arquivo `.proto` detectado no projeto.\n");
        return s;
    }

    s.push_str("## Files\n\n");
    for f in &report.protobuf.files_found {
        s.push_str(&format!(
            "- **{}** package `{}` ({} services, {} messages)\n",
            f.path,
            f.package.as_deref().unwrap_or("?"),
            f.service_count,
            f.message_count
        ));
    }
    s.push('\n');

    s.push_str("## Services and RPCs\n\n");
    for svc in &report.protobuf.services {
        s.push_str(&format!("### `{}` in `{}`\n\n", svc.name, svc.file));
        if !svc.rpcs.is_empty() {
            s.push_str("| RPC | Request | Response | Streaming | Status |\n");
            s.push_str("|-----|---------|----------|-----------|--------|\n");
            for rpc in &svc.rpcs {
                let status = report
                    .crossref
                    .items
                    .iter()
                    .find(|i| {
                        matches!(&i.source, SchemaSource::Protobuf { service, .. } if service == &svc.name)
                            && i.identifier == format!("{}.{}", svc.name, rpc.name)
                    })
                    .map(|i| format!("{:?}", i.status).to_uppercase())
                    .unwrap_or_else(|| "?".to_string());
                s.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {:?} | {} |\n",
                    rpc.name, rpc.request_type, rpc.response_type, rpc.streaming, status
                ));
            }
            s.push('\n');
        }
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar**: mesma semantica da OpenAPI. PHANTOM RPC significa spec declara mas codigo nao implementa handler. Priorizar implementacao antes de mudar spec.\n");
    s
}

fn render_graphql_md(report: &ContractsReport) -> String {
    let mut s = String::new();
    s.push_str("# GraphQL Contracts\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine contracts` at {}\n\n",
        report.generated_at
    ));

    if report.graphql.schemas_found.is_empty() {
        s.push_str("Nenhum schema GraphQL (`.graphql` / `.gql`) encontrado.\n");
        return s;
    }

    s.push_str("## Schemas\n\n");
    for sch in &report.graphql.schemas_found {
        s.push_str(&format!(
            "- **{}** ({} operations, {} types)\n",
            sch.path, sch.operation_count, sch.type_count
        ));
    }
    s.push('\n');

    s.push_str("## Operations (resolvers esperados)\n\n");
    s.push_str("| Kind | Name | Status |\n");
    s.push_str("|------|------|--------|\n");
    for op in &report.graphql.operations {
        let status = report
            .crossref
            .items
            .iter()
            .find(|i| matches!(&i.source, SchemaSource::Graphql { .. }) && i.identifier == op.name)
            .map(|i| format!("{:?}", i.status).to_uppercase())
            .unwrap_or_else(|| "?".to_string());
        s.push_str(&format!("| {:?} | `{}` | {} |\n", op.kind, op.name, status));
    }
    s.push('\n');

    s.push_str("---\n\n");
    s.push_str("**Como usar**: cada operation top-level (Query/Mutation/Subscription field) deveria ter resolver correspondente no codigo. PHANTOM = resolver ausente, geralmente causa runtime error.\n");
    s
}

fn render_drift_md(report: &ContractsReport) -> String {
    let mut s = String::new();
    s.push_str("# Contract Drift Summary\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine contracts` at {}\n\n",
        report.generated_at
    ));

    let s_sum = &report.crossref.summary;
    s.push_str("## Overview\n\n");
    s.push_str(&format!("- Total contract entities: {}\n", s_sum.total));
    s.push_str(&format!(
        "- **Implemented** (forte evidencia): {}\n",
        s_sum.implemented
    ));
    s.push_str(&format!(
        "- **Candidates** (evidencia fraca, requer validacao humana): {}\n",
        s_sum.candidates
    ));
    s.push_str(&format!(
        "- **Phantoms** (spec declara mas codigo nao implementa): {}\n\n",
        s_sum.phantoms
    ));

    let phantoms: Vec<_> = report
        .crossref
        .items
        .iter()
        .filter(|i| matches!(i.status, CrossrefStatus::Phantom))
        .collect();

    if !phantoms.is_empty() {
        s.push_str(&format!("## Phantoms ({})\n\n", phantoms.len()));
        s.push_str(
            "Entidades declaradas em spec mas sem evidencia de implementacao no codigo.\n\n",
        );
        s.push_str("| Identifier | Source |\n");
        s.push_str("|------------|--------|\n");
        for p in &phantoms {
            s.push_str(&format!(
                "| `{}` | {} |\n",
                p.identifier,
                source_label(&p.source)
            ));
        }
        s.push('\n');
    }

    let candidates: Vec<_> = report
        .crossref
        .items
        .iter()
        .filter(|i| matches!(i.status, CrossrefStatus::Candidate))
        .collect();
    if !candidates.is_empty() {
        s.push_str(&format!("## Candidates ({})\n\n", candidates.len()));
        s.push_str("Evidencia fraca (1-2 matches). Pode ser implementacao real com naming diferente ou pode ser phantom.\n\n");
        s.push_str("| Identifier | Source | Evidence |\n");
        s.push_str("|------------|--------|----------|\n");
        for c in candidates.iter().take(30) {
            let ev_preview = c
                .evidence
                .iter()
                .take(2)
                .map(|e| format!("`{}:{}`", e.file, e.line))
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!(
                "| `{}` | {} | {} |\n",
                c.identifier,
                source_label(&c.source),
                ev_preview
            ));
        }
        s.push('\n');
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar**: phantoms indicam gap entre spec e implementacao - decidir se remove da spec ou implementa. Candidates precisam olho humano - naming heuristico pode ser confundido por identificadores similares. Implemented sao referencias confirmadas.\n");
    s
}
