use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_warning,
    OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::runtime::{analyze, RuntimeReport};
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
        .unwrap_or_else(|| args.root.join(".first-plan").join("14-runtime"));

    std::fs::create_dir_all(&out_dir)?;
    std::fs::write(out_dir.join("00-releases.md"), render_releases_md(&report))?;
    std::fs::write(
        out_dir.join("01-file-releases.md"),
        render_file_releases_md(&report),
    )?;
    std::fs::write(
        out_dir.join("02-unreleased.md"),
        render_unreleased_md(&report),
    )?;
    std::fs::write(out_dir.join("03-summary.md"), render_summary_md(&report))?;
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

fn render_pretty(report: &RuntimeReport, out_dir: &std::path::Path) {
    print_header(&format!("Runtime Layer ({}ms)", report.elapsed_ms));

    print_section("Releases");
    if report.releases.total_releases == 0 {
        print_warning("Nenhuma release tag detectada");
    } else {
        print_kv_bold(
            "Total releases",
            &report.releases.total_releases.to_string(),
            Color::Green,
        );
        if let Some(latest) = &report.releases.latest_tag {
            print_kv("Latest tag", latest, Color::White);
        }
        print_kv(
            "CHANGELOG matched",
            &format!(
                "{}/{}",
                report.releases.changelog_matched, report.releases.total_releases
            ),
            Color::DarkGrey,
        );
        for release in report.releases.releases.iter().rev().take(5) {
            let ver_color = if release.is_semver {
                Color::Green
            } else {
                Color::Yellow
            };
            println!(
                "  {} {} {} {}",
                release.tag.as_str().bold().with(ver_color),
                release.date.as_str().dim(),
                format!("{} commits", release.commit_count).with(Color::White),
                format!("{} authors", release.author_count).dim()
            );
        }
    }

    print_section("Unreleased (post latest tag)");
    let u = &report.unreleased;
    if u.commits_count == 0 {
        print_warning("Nenhum commit pendente - main esta em sync com latest tag");
    } else {
        let color = if u.commits_count > 20 {
            Color::Red
        } else if u.commits_count > 5 {
            Color::Yellow
        } else {
            Color::Green
        };
        print_kv_bold("Commits pending", &u.commits_count.to_string(), color);
        if let Some(since) = &u.since_tag {
            print_kv("Since tag", since, Color::DarkGrey);
        }
        if u.has_breaking {
            print_kv_bold("Breaking changes", "YES", Color::Red);
        }
        print_kv(
            "Files touched",
            &u.files_touched.len().to_string(),
            Color::White,
        );
        print_kv("Authors", &u.authors.len().to_string(), Color::White);
        for commit in u.commits.iter().take(5) {
            let marker = if commit.is_breaking { "!" } else { " " };
            println!(
                "  {}{} {} {}",
                marker.with(Color::Red),
                commit.short_sha.as_str().with(Color::DarkGrey),
                commit.subject.as_str().bold(),
                format!("({})", commit.author).dim()
            );
        }
    }

    print_section("File releases");
    let fr = &report.file_releases;
    if fr.total_files_analyzed == 0 {
        print_warning("Sem arquivos ou sem releases para mapear");
    } else {
        print_kv_bold(
            "Files analyzed",
            &fr.total_files_analyzed.to_string(),
            Color::Green,
        );
        let unreleased_count = fr.files.iter().filter(|f| f.is_unreleased).count();
        print_kv(
            "Unreleased files",
            &unreleased_count.to_string(),
            if unreleased_count > 0 {
                Color::Yellow
            } else {
                Color::DarkGrey
            },
        );
        if !fr.introduced_by_release.is_empty() {
            println!();
            println!("  {}", "Top releases by file introductions:".dim());
            for rel in fr.introduced_by_release.iter().take(5) {
                println!(
                    "    {} {} files",
                    rel.release.as_str().bold().with(Color::Cyan),
                    rel.file_count
                );
            }
        }
    }

    println!();
    print_kv_bold("Saved to", &out_dir.to_string_lossy(), Color::Green);
    flush();
}

fn render_releases_md(report: &RuntimeReport) -> String {
    let mut s = String::new();
    s.push_str("# Release History\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine runtime` at {}\n\n",
        report.generated_at
    ));

    if report.releases.total_releases == 0 {
        s.push_str("Nenhuma release tag encontrada. Este projeto ainda nao usa git tags para releases.\n\n");
        s.push_str("Recomendado adotar tags semver (`v1.0.0`, `v0.10.0`) para runtime awareness completa.\n");
        return s;
    }

    s.push_str(&format!(
        "**Total releases**: {}\n",
        report.releases.total_releases
    ));
    if let Some(latest) = &report.releases.latest_tag {
        s.push_str(&format!("**Latest**: `{}`\n", latest));
    }
    s.push_str(&format!(
        "**CHANGELOG matched**: {}/{} releases\n\n",
        report.releases.changelog_matched, report.releases.total_releases
    ));

    s.push_str("## Timeline (newest first)\n\n");
    s.push_str("| Tag | Date | Commits | Authors | Semver | CHANGELOG |\n");
    s.push_str("|-----|------|---------|---------|--------|-----------|\n");
    for r in report.releases.releases.iter().rev() {
        let cl = r.changelog_entry.as_deref().unwrap_or("-");
        s.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            r.tag,
            r.date,
            r.commit_count,
            r.author_count,
            if r.is_semver { "yes" } else { "no" },
            cl
        ));
    }
    s.push('\n');

    s.push_str("---\n\n");
    s.push_str("**Como usar**: cada release define snapshot do codebase em ponto no tempo. AI ao propor mudanca em codigo pode conferir em qual release o codigo alvo mora - codigo em releases antigas eh production-stable, codigo introduzido recentemente eh mais risco.\n");
    s
}

fn render_unreleased_md(report: &RuntimeReport) -> String {
    let mut s = String::new();
    s.push_str("# Unreleased Changes\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine runtime` at {}\n\n",
        report.generated_at
    ));

    let u = &report.unreleased;

    if u.commits_count == 0 {
        s.push_str(
            "**Estado limpo**: main esta em sync com latest release tag. Nenhum commit pendente.\n",
        );
        return s;
    }

    if let Some(since) = &u.since_tag {
        s.push_str(&format!("**Since**: `{}`\n", since));
    }
    s.push_str(&format!("**Commits pending**: {}\n", u.commits_count));
    s.push_str(&format!("**Authors**: {}\n", u.authors.len()));
    s.push_str(&format!("**Files touched**: {}\n", u.files_touched.len()));
    if u.has_breaking {
        s.push_str("**Breaking changes**: YES (feat!, fix!, BREAKING CHANGE)\n");
    }
    s.push('\n');

    if !u.commits.is_empty() {
        s.push_str("## Commits (up to 50)\n\n");
        s.push_str("| SHA | Date | Author | Subject | Breaking |\n");
        s.push_str("|-----|------|--------|---------|----------|\n");
        for c in &u.commits {
            let date_short = c.date.get(..10).unwrap_or(&c.date);
            s.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                c.short_sha,
                date_short,
                c.author,
                c.subject,
                if c.is_breaking { "!" } else { "" }
            ));
        }
        s.push('\n');
    }

    if !u.authors.is_empty() {
        s.push_str("## Authors contribution\n\n");
        for a in &u.authors {
            s.push_str(&format!("- {} ({} commits)\n", a.name, a.commit_count));
        }
        s.push('\n');
    }

    if !u.files_touched.is_empty() {
        s.push_str("## Files most touched (top 20)\n\n");
        for f in u.files_touched.iter().take(20) {
            s.push_str(&format!("- `{}` ({}x)\n", f.path, f.touches));
        }
        s.push('\n');
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar**: se AI propoe fix em bug de producao, conferir se area alvo tem commits unreleased. Se sim, fix pode ir junto na proxima release. Breaking changes pending sinalizam que proxima release sera major bump.\n");
    s
}

fn render_file_releases_md(report: &RuntimeReport) -> String {
    let mut s = String::new();
    s.push_str("# File Release Map\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine runtime` at {}\n\n",
        report.generated_at
    ));

    let fr = &report.file_releases;

    if fr.total_files_analyzed == 0 {
        s.push_str("Sem arquivos source encontrados ou sem releases para mapear.\n");
        return s;
    }

    let unreleased_files: Vec<_> = fr.files.iter().filter(|f| f.is_unreleased).collect();
    let released_files: Vec<_> = fr.files.iter().filter(|f| !f.is_unreleased).collect();

    s.push_str(&format!(
        "**Total files analyzed**: {}\n",
        fr.total_files_analyzed
    ));
    s.push_str(&format!(
        "**Released (last mod in some release)**: {}\n",
        released_files.len()
    ));
    s.push_str(&format!(
        "**Unreleased (last mod only in main)**: {}\n\n",
        unreleased_files.len()
    ));

    if !fr.introduced_by_release.is_empty() {
        s.push_str("## Files introduced per release\n\n");
        s.push_str("| Release | New files |\n");
        s.push_str("|---------|-----------|\n");
        for r in &fr.introduced_by_release {
            s.push_str(&format!("| `{}` | {} |\n", r.release, r.file_count));
        }
        s.push('\n');
    }

    if !unreleased_files.is_empty() {
        s.push_str(&format!(
            "## Unreleased files ({}) - editable/removable com maior liberdade\n\n",
            unreleased_files.len()
        ));
        s.push_str(
            "Estes arquivos foram modificados apenas em main sem alcancar release ainda.\n\n",
        );
        s.push_str("| Path | Introduced | Commits |\n");
        s.push_str("|------|------------|---------|\n");
        for f in unreleased_files.iter().take(30) {
            s.push_str(&format!(
                "| `{}` | {} | {} |\n",
                f.path,
                f.introduced_in.as_deref().unwrap_or("main-only"),
                f.commit_count
            ));
        }
        if unreleased_files.len() > 30 {
            s.push_str(&format!(
                "\n_({} mais em `report.json`)_\n",
                unreleased_files.len() - 30
            ));
        }
        s.push('\n');
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar**: arquivos released ha muito tempo sao production-stable - mudancas requerem cuidado extra (backwards compat). Arquivos unreleased sao work-in-progress, editaveis com menos risco.\n");
    s
}

fn render_summary_md(report: &RuntimeReport) -> String {
    let mut s = String::new();
    s.push_str("# Runtime Summary\n\n");
    s.push_str(&format!(
        "Generated by `first-plan-engine runtime` at {}\n\n",
        report.generated_at
    ));

    s.push_str("## Release state\n\n");
    if report.releases.total_releases == 0 {
        s.push_str("- Nenhuma release tag detectada\n");
        s.push_str("- Recomendado adotar tags semver antes de continuar\n\n");
    } else {
        s.push_str(&format!(
            "- Total releases: {}\n",
            report.releases.total_releases
        ));
        if let Some(latest) = &report.releases.latest_tag {
            s.push_str(&format!("- Latest: `{}`\n", latest));
        }
        s.push_str(&format!(
            "- CHANGELOG entries matched: {}/{}\n\n",
            report.releases.changelog_matched, report.releases.total_releases
        ));
    }

    s.push_str("## Unreleased state\n\n");
    let u = &report.unreleased;
    if u.commits_count == 0 {
        s.push_str("- Estado limpo: main == latest tag\n\n");
    } else {
        s.push_str(&format!("- {} commits pending release\n", u.commits_count));
        s.push_str(&format!("- {} authors contributing\n", u.authors.len()));
        s.push_str(&format!("- {} files touched\n", u.files_touched.len()));
        if u.has_breaking {
            s.push_str("- **BREAKING CHANGES pending** - proxima release sera major/breaking\n");
        }
        s.push('\n');
    }

    let unreleased_files = report
        .file_releases
        .files
        .iter()
        .filter(|f| f.is_unreleased)
        .count();
    if unreleased_files > 0 {
        s.push_str(&format!(
            "## {} arquivos em estado unreleased\n\n",
            unreleased_files
        ));
        s.push_str("Ver `01-file-releases.md` para lista completa.\n\n");
    }

    s.push_str("---\n\n");
    s.push_str("**Como usar em Plan-First**: antes de propor mudanca, ler este summary para saber se codigo alvo esta released (production-stable) ou unreleased (edge). Fix em bug de producao deve ir em area released ou aguardar release. Mudanca breaking pendente influencia timing da proxima release.\n");
    s
}
