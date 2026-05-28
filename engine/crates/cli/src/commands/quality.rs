use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_warning,
    OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::quality::{analyze, ci, coverage, flaky, QualityReport};
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Diretorio onde escrever os 3 markdown files. Default: .first-plan/11-quality/
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Imprime JSON completo em stdout em vez do pretty output.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let mode = output_mode(args.json);
    let report = analyze(&args.root);

    let out_dir = args
        .output
        .clone()
        .unwrap_or_else(|| args.root.join(".first-plan").join("11-quality"));

    std::fs::create_dir_all(&out_dir)?;
    std::fs::write(out_dir.join("00-pipeline.md"), render_pipeline_md(&report))?;
    std::fs::write(out_dir.join("01-coverage.md"), render_coverage_md(&report))?;
    std::fs::write(out_dir.join("02-flaky.md"), render_flaky_md(&report))?;
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

fn render_pretty(report: &QualityReport, out_dir: &std::path::Path) {
    print_header(&format!("Quality Layer ({}ms)", report.elapsed_ms));

    print_section("CI");
    if report.ci.providers_detected.is_empty() {
        print_warning("Nenhum provider de CI detectado");
    } else {
        print_kv(
            "Providers",
            &report.ci.providers_detected.join(", "),
            Color::White,
        );
        print_kv_bold(
            "Total jobs",
            &report.ci.total_jobs.to_string(),
            Color::Green,
        );
        let triggers = &report.ci.triggers_summary;
        let mut active = Vec::new();
        if triggers.on_push {
            active.push("push");
        }
        if triggers.on_pull_request {
            active.push("PR");
        }
        if triggers.on_schedule {
            active.push("schedule");
        }
        if triggers.on_release {
            active.push("release");
        }
        if triggers.on_workflow_dispatch {
            active.push("manual");
        }
        print_kv("Triggers", &active.join(", "), Color::White);
        for wf in &report.ci.workflows {
            let label = wf.name.as_deref().unwrap_or(&wf.file);
            println!(
                "  {} {} {}",
                wf.provider.as_str().with(Color::Cyan),
                label.bold(),
                format!("({} jobs)", wf.jobs.len()).dim()
            );
        }
    }

    print_section("Coverage");
    if report.coverage.formats_detected.is_empty() {
        print_warning("Nenhum coverage report encontrado");
    } else {
        print_kv(
            "Formats",
            &report.coverage.formats_detected.join(", "),
            Color::White,
        );
        print_kv_bold(
            "Files",
            &report.coverage.overall.files_count.to_string(),
            Color::Green,
        );
        let overall_color = if report.coverage.overall.percent >= 80.0 {
            Color::Green
        } else if report.coverage.overall.percent >= 50.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        print_kv_bold(
            "Overall",
            &format!("{:.1}%", report.coverage.overall.percent),
            overall_color,
        );
        let worst_count = report.coverage.source_files.len().min(5);
        if worst_count > 0 {
            println!("  {} worst-covered files:", "Top".dim());
            for f in &report.coverage.source_files[..worst_count] {
                let color = if f.percent >= 80.0 {
                    Color::Green
                } else if f.percent >= 50.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                println!(
                    "    {} {} {} {}",
                    format!("{:>5.1}%", f.percent).with(color),
                    f.path.as_str().with(Color::White),
                    format!("({}/{} lines)", f.lines_covered, f.lines_total).dim(),
                    if !f.uncovered_ranges.is_empty() {
                        format!("[{} gaps]", f.uncovered_ranges.len()).dim()
                    } else {
                        String::new().dim()
                    }
                );
            }
        }
    }

    print_section("Flaky tests");
    if report.flaky.analyzed_commits == 0 {
        print_warning("Repo nao tem historia git suficiente");
    } else {
        print_kv(
            "Window",
            &format!("{} days", report.flaky.window_days),
            Color::DarkGrey,
        );
        print_kv(
            "Commits analyzed",
            &report.flaky.analyzed_commits.to_string(),
            Color::DarkGrey,
        );
        print_kv_bold(
            "Candidates",
            &report.flaky.candidates.len().to_string(),
            if report.flaky.candidates.is_empty() {
                Color::Green
            } else {
                Color::Yellow
            },
        );
        let top = report.flaky.candidates.len().min(5);
        for c in &report.flaky.candidates[..top] {
            let score_color = if c.score >= 1.0 {
                Color::Red
            } else if c.score >= 0.5 {
                Color::Yellow
            } else {
                Color::DarkGrey
            };
            println!(
                "  {} {}",
                format!("{:.2}", c.score).with(score_color),
                c.path.as_str().bold().with(Color::Cyan)
            );
            for s in &c.signals {
                println!("        {}", s.as_str().dim());
            }
        }
    }

    println!();
    print_kv_bold("Saved to", &out_dir.to_string_lossy(), Color::Green);
    flush();
}

fn render_pipeline_md(report: &QualityReport) -> String {
    let mut s = String::new();
    s.push_str("# Pipeline (CI/CD)\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine quality` at {}\n\n",
        report.generated_at
    ));

    if report.ci.providers_detected.is_empty() {
        s.push_str("Nenhum provider de CI detectado neste projeto.\n");
        return s;
    }

    s.push_str(&format!(
        "**Providers**: {}\n\n",
        report.ci.providers_detected.join(", ")
    ));

    let triggers = &report.ci.triggers_summary;
    s.push_str("## Triggers\n\n");
    let mut bullets = Vec::new();
    if triggers.on_push {
        bullets.push("- `push` (CI valida toda mudanca em branch)");
    }
    if triggers.on_pull_request {
        bullets.push("- `pull_request` (CI valida antes de merge)");
    }
    if triggers.on_schedule {
        bullets.push("- `schedule` / `cron` (CI roda periodicamente)");
    }
    if triggers.on_release {
        bullets.push("- `release` (CI roda em tags/releases)");
    }
    if triggers.on_workflow_dispatch {
        bullets.push("- `workflow_dispatch` / `manual` (CI pode ser invocado manualmente)");
    }
    if bullets.is_empty() && triggers.on_other.is_empty() {
        s.push_str("Sem triggers detectaveis.\n\n");
    } else {
        for b in bullets {
            s.push_str(b);
            s.push('\n');
        }
        for other in &triggers.on_other {
            s.push_str(&format!("- `{}`\n", other));
        }
        s.push('\n');
    }

    s.push_str("## Workflows\n\n");
    for wf in &report.ci.workflows {
        let title = wf.name.as_deref().unwrap_or(&wf.file);
        s.push_str(&format!("### {} ({})\n\n", title, wf.provider));
        s.push_str(&format!("- File: `{}`\n", wf.file));
        s.push_str(&format!("- Triggers: {}\n", wf.triggers.join(", ")));
        s.push_str(&format!("- Jobs: {}\n\n", wf.jobs.len()));

        if !wf.jobs.is_empty() {
            s.push_str("| Job | Runs on | Steps |\n");
            s.push_str("|-----|---------|-------|\n");
            for job in &wf.jobs {
                s.push_str(&format!(
                    "| `{}` | {} | {} |\n",
                    job.name,
                    job.runs_on.as_deref().unwrap_or("?"),
                    job.steps_count
                ));
            }
            s.push('\n');
        }
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar este IR**: ao planejar mudanca, conferir quais jobs rodam (push vs PR vs release). Mudanca que afeta apenas codigo de teste pode ainda assim quebrar workflow se step `cargo clippy -- -D warnings` for estrito.\n");
    s
}

fn render_coverage_md(report: &QualityReport) -> String {
    let mut s = String::new();
    s.push_str("# Test Coverage\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine quality` at {}\n\n",
        report.generated_at
    ));

    if report.coverage.formats_detected.is_empty() {
        s.push_str("Nenhum coverage report encontrado. Rodar testes com:\n\n");
        s.push_str("- Rust: `cargo tarpaulin --out lcov` ou `cargo llvm-cov --lcov`\n");
        s.push_str("- Node: `jest --coverage` (gera coverage/coverage-summary.json)\n");
        s.push_str("- Go: `go test -coverprofile=coverage.out ./...`\n");
        s.push_str("- Python: `pytest --cov --cov-report=xml`\n");
        s.push_str("- Java: surefire/jacoco-maven-plugin\n");
        return s;
    }

    s.push_str(&format!(
        "**Formats**: {}\n\n",
        report.coverage.formats_detected.join(", ")
    ));

    let stats = &report.coverage.overall;
    s.push_str("## Overall\n\n");
    s.push_str(&format!("- Files analyzed: {}\n", stats.files_count));
    s.push_str(&format!("- Lines total: {}\n", stats.lines_total));
    s.push_str(&format!("- Lines covered: {}\n", stats.lines_covered));
    s.push_str(&format!("- **Overall: {:.1}%**\n\n", stats.percent));

    s.push_str("## Worst-covered files (top 20)\n\n");
    s.push_str("| Path | Coverage | Lines | Uncovered ranges |\n");
    s.push_str("|------|----------|-------|------------------|\n");
    for f in report.coverage.source_files.iter().take(20) {
        let ranges_str = if f.uncovered_ranges.is_empty() {
            "-".to_string()
        } else {
            let preview: Vec<String> = f
                .uncovered_ranges
                .iter()
                .take(3)
                .map(|r| {
                    if r.start == r.end {
                        r.start.to_string()
                    } else {
                        format!("{}-{}", r.start, r.end)
                    }
                })
                .collect();
            let suffix = if f.uncovered_ranges.len() > 3 {
                format!(" (+{})", f.uncovered_ranges.len() - 3)
            } else {
                String::new()
            };
            format!("{}{}", preview.join(", "), suffix)
        };
        s.push_str(&format!(
            "| `{}` | {:.1}% | {}/{} | {} |\n",
            f.path, f.percent, f.lines_covered, f.lines_total, ranges_str
        ));
    }
    s.push('\n');

    if report.coverage.source_files.len() > 20 {
        s.push_str(&format!(
            "_({} more files in `report.json`)_\n\n",
            report.coverage.source_files.len() - 20
        ));
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar este IR**: antes de refatorar arquivo, verificar coverage. Arquivos < 50% sao alto risco - mudanca pode quebrar comportamento nao testado. Linhas no ranges sao gaps especificos - refactor ali tem fall-back zero.\n");
    s
}

fn render_flaky_md(report: &QualityReport) -> String {
    let mut s = String::new();
    s.push_str("# Flaky Tests\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine quality` at {}\n\n",
        report.generated_at
    ));

    if report.flaky.analyzed_commits == 0 {
        s.push_str("Repo sem historia git suficiente para detectar flaky tests.\n");
        return s;
    }

    s.push_str(&format!(
        "**Window**: ultimos {} dias ({} commits analisados)\n\n",
        report.flaky.window_days, report.flaky.analyzed_commits
    ));

    if report.flaky.candidates.is_empty() {
        s.push_str("Nenhum candidato a flaky test detectado. Heuristicas usadas:\n\n");
        s.push_str(
            "- Test file editado isoladamente com keyword suspeita (flaky/race/timeout/etc)\n",
        );
        s.push_str("- Test file mencionado em commits revert\n");
        s.push_str("- Test file com alta frequencia de edits isolados sem mudanca em codigo correspondente\n\n");
        s.push_str("Ausencia de candidatos eh sinal positivo mas nao garantia - flaky tests podem ser silenciosos.\n");
        return s;
    }

    s.push_str(&format!(
        "## Candidatos detectados ({})\n\n",
        report.flaky.candidates.len()
    ));

    s.push_str("Ordenado por score (alto = forte evidencia de instabilidade).\n\n");
    s.push_str("| Score | Path | Sinais |\n");
    s.push_str("|-------|------|--------|\n");
    for c in &report.flaky.candidates {
        let signals = c.signals.join("; ");
        s.push_str(&format!(
            "| **{:.2}** | `{}` | {} |\n",
            c.score, c.path, signals
        ));
    }
    s.push('\n');

    s.push_str("---\n\n");
    s.push_str("**Como usar este IR**: ao avaliar PRs ou planejar refatoracoes, tratar flaky tests com cuidado. Refatorar codigo testado por flaky test pode mascarar regressao real. Tests com score alto sao candidatos a investigacao + correcao de fonte do flake (race condition, timeout, ordem de teste, etc).\n\n");
    s.push_str("**Limites da heuristica**: deteccao baseada em git history. Falsos positivos possiveis quando time refatora tests legitimamente. Falsos negativos em flakes silenciosos que nunca foram diagnosticados.\n");
    s
}

#[allow(dead_code)]
fn _suppress_unused(_a: &ci::CiReport, _b: &coverage::CoverageReport, _c: &flaky::FlakyReport) {}
