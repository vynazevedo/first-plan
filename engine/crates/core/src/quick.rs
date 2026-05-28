//! Quick glance - overview do projeto em <60s.
//!
//! Output denso para /fp:quick: stacks detectadas, entry points, top simbolos,
//! atividade git, convencoes basicas e comandos uteis. Sem tree-sitter parsing
//! profundo, sem indexing, sem subagents - tudo via heuristica + walk + git CLI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlanceReport {
    pub generated_at: String,
    pub elapsed_ms: u64,
    pub root: String,
    pub stacks: Vec<StackHint>,
    pub entry_points: Vec<String>,
    pub top_symbols: Vec<SymbolHint>,
    pub git_activity: Option<GitActivity>,
    pub conventions: ConventionHints,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHint {
    pub language: String,
    pub manifest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolHint {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitActivity {
    pub recent_commits: Vec<String>,
    pub hot_files: Vec<HotFile>,
    pub active_authors: Vec<AuthorActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotFile {
    pub path: String,
    pub change_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorActivity {
    pub name: String,
    pub commit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionHints {
    pub naming: Option<String>,
    pub test_frameworks: Vec<String>,
}

pub fn glance(root: &Path) -> GlanceReport {
    let start = Instant::now();
    let stacks = detect_stacks(root);
    let entry_points = detect_entry_points(root);
    let top_symbols = sample_symbols(root, 25);
    let git_activity = git_summary(root);
    let conventions = sample_conventions(root);
    let commands = suggest_commands(&stacks, root);

    GlanceReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        root: root.to_string_lossy().into_owned(),
        stacks,
        entry_points,
        top_symbols,
        git_activity,
        conventions,
        commands,
    }
}

fn detect_stacks(root: &Path) -> Vec<StackHint> {
    let manifests: &[(&str, &str)] = &[
        ("Cargo.toml", "Rust"),
        ("go.mod", "Go"),
        ("package.json", "JavaScript/TypeScript"),
        ("tsconfig.json", "TypeScript"),
        ("pyproject.toml", "Python"),
        ("setup.py", "Python"),
        ("requirements.txt", "Python"),
        ("Pipfile", "Python"),
        ("composer.json", "PHP"),
        ("Gemfile", "Ruby"),
        ("Makefile", "Make"),
        ("CMakeLists.txt", "C/C++"),
        ("compile_commands.json", "C/C++"),
        (".luarc.json", "Lua"),
        ("pubspec.yaml", "Dart/Flutter"),
        ("mix.exs", "Elixir"),
        ("build.gradle", "Java/Kotlin"),
        ("build.gradle.kts", "Java/Kotlin"),
        ("pom.xml", "Java"),
        ("*.csproj", "C#/.NET"),
        ("Package.swift", "Swift"),
    ];

    let mut out = Vec::new();
    let mut seen_langs = std::collections::HashSet::new();

    let search_dirs: Vec<PathBuf> = std::iter::once(root.to_path_buf())
        .chain(
            std::fs::read_dir(root)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    ![
                        "target",
                        "node_modules",
                        "vendor",
                        "dist",
                        "build",
                        ".git",
                        ".cache",
                    ]
                    .contains(&n.as_str())
                        && !n.starts_with('.')
                })
                .map(|e| e.path()),
        )
        .collect();

    for dir in &search_dirs {
        for (manifest, lang) in manifests {
            if let Some(suffix) = manifest.strip_prefix('*') {
                if let Ok(rd) = std::fs::read_dir(dir) {
                    for entry in rd.flatten() {
                        if let Some(n) = entry.file_name().to_str() {
                            if n.ends_with(suffix) && seen_langs.insert(lang.to_string()) {
                                let rel = entry
                                    .path()
                                    .strip_prefix(root)
                                    .unwrap_or(entry.path().as_path())
                                    .to_string_lossy()
                                    .into_owned();
                                out.push(StackHint {
                                    language: lang.to_string(),
                                    manifest: rel,
                                });
                            }
                        }
                    }
                }
            } else if dir.join(manifest).exists() && seen_langs.insert(lang.to_string()) {
                let abs = dir.join(manifest);
                let rel = abs
                    .strip_prefix(root)
                    .unwrap_or(abs.as_path())
                    .to_string_lossy()
                    .into_owned();
                out.push(StackHint {
                    language: lang.to_string(),
                    manifest: rel,
                });
            }
        }
    }

    out
}

fn detect_entry_points(root: &Path) -> Vec<String> {
    let candidates = [
        "main.rs",
        "main.go",
        "main.py",
        "app.py",
        "index.ts",
        "index.js",
        "index.tsx",
        "index.jsx",
        "server.js",
        "server.ts",
        "Main.java",
        "main.kt",
        "Program.cs",
        "Application.swift",
    ];

    let mut found = Vec::new();
    let exclude = ["target", "node_modules", "vendor", "dist", "build", ".git"];

    'outer: for entry in WalkDir::new(root)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !exclude.iter().any(|x| name == *x)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if candidates.contains(&name.as_str()) {
            if let Ok(rel) = entry.path().strip_prefix(root) {
                found.push(rel.to_string_lossy().into_owned());
                if found.len() >= 10 {
                    break 'outer;
                }
            }
        }
    }
    found
}

fn sample_symbols(root: &Path, max: usize) -> Vec<SymbolHint> {
    let patterns: &[(&str, &str)] = &[
        ("pub fn ", "function"),
        ("pub struct ", "struct"),
        ("pub enum ", "enum"),
        ("pub trait ", "trait"),
        ("func ", "function"),
        ("type ", "type"),
        ("def ", "function"),
        ("class ", "class"),
        ("export function ", "function"),
        ("export class ", "class"),
        ("export interface ", "interface"),
        ("export type ", "type"),
        ("interface ", "interface"),
    ];

    let exts = [
        "rs", "go", "py", "ts", "tsx", "js", "jsx", "java", "kt", "rb",
    ];
    let exclude = ["target", "node_modules", "vendor", "dist", "build", ".git"];

    let mut out = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !exclude.iter().any(|x| name == *x)
        })
        .filter_map(|e| e.ok())
    {
        if out.len() >= max {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !exts.contains(&ext) {
            continue;
        }

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) if c.len() < 500_000 => c,
            _ => continue,
        };

        let rel = entry
            .path()
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        for (idx, line) in content.lines().enumerate().take(500) {
            for (kw, kind) in patterns {
                if let Some(pos) = line.find(kw) {
                    let after = &line[pos + kw.len()..];
                    let name: String = after
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if name.is_empty() || name.starts_with('_') {
                        continue;
                    }
                    out.push(SymbolHint {
                        name,
                        kind: kind.to_string(),
                        file: rel.clone(),
                        line: (idx + 1) as u32,
                    });
                    break;
                }
            }
            if out.len() >= max {
                break;
            }
        }
    }
    out
}

fn git_summary(root: &Path) -> Option<GitActivity> {
    if !root.join(".git").exists() {
        return None;
    }

    let commits_out = Command::new("git")
        .args(["log", "--oneline", "-n", "10"])
        .current_dir(root)
        .output()
        .ok()?;
    let recent_commits: Vec<String> = String::from_utf8_lossy(&commits_out.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let names_out = Command::new("git")
        .args([
            "log",
            "--since=90 days ago",
            "--name-only",
            "--pretty=format:",
        ])
        .current_dir(root)
        .output()
        .ok()?;
    let mut file_counts: HashMap<String, u32> = HashMap::new();
    for line in String::from_utf8_lossy(&names_out.stdout).lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *file_counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }
    }
    let mut hot: Vec<(String, u32)> = file_counts.into_iter().collect();
    hot.sort_by_key(|h| std::cmp::Reverse(h.1));
    hot.truncate(5);
    let hot_files: Vec<HotFile> = hot
        .into_iter()
        .map(|(path, change_count)| HotFile { path, change_count })
        .collect();

    let authors_out = Command::new("git")
        .args(["log", "--since=90 days ago", "--pretty=format:%an"])
        .current_dir(root)
        .output()
        .ok()?;
    let mut author_counts: HashMap<String, u32> = HashMap::new();
    for line in String::from_utf8_lossy(&authors_out.stdout).lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *author_counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }
    }
    let mut authors: Vec<(String, u32)> = author_counts.into_iter().collect();
    authors.sort_by_key(|a| std::cmp::Reverse(a.1));
    authors.truncate(5);
    let active_authors: Vec<AuthorActivity> = authors
        .into_iter()
        .map(|(name, commit_count)| AuthorActivity { name, commit_count })
        .collect();

    Some(GitActivity {
        recent_commits,
        hot_files,
        active_authors,
    })
}

fn sample_conventions(root: &Path) -> ConventionHints {
    let exclude = ["target", "node_modules", "vendor", "dist", "build", ".git"];
    let mut snake = 0u32;
    let mut camel = 0u32;
    let mut kebab = 0u32;
    let mut sample = 0u32;

    let mut frameworks = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !exclude.iter().any(|x| name == *x)
        })
        .filter_map(|e| e.ok())
    {
        if sample >= 100 {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let stem = match entry.path().file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !["go", "rs", "py", "ts", "js", "tsx", "jsx", "rb"].contains(&ext) {
            continue;
        }

        if stem.contains('-') {
            kebab += 1;
        } else if stem.contains('_') {
            snake += 1;
        } else if stem.chars().any(|c| c.is_ascii_uppercase())
            && stem
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false)
        {
            camel += 1;
        } else if !stem.is_empty() {
            snake += 1;
        }
        sample += 1;
    }

    let naming = if snake + camel + kebab == 0 {
        None
    } else if snake >= camel && snake >= kebab {
        Some(format!(
            "snake_case predominante ({} de {} arquivos)",
            snake,
            snake + camel + kebab
        ))
    } else if camel > kebab {
        Some(format!(
            "camelCase predominante ({} de {} arquivos)",
            camel,
            snake + camel + kebab
        ))
    } else {
        Some(format!(
            "kebab-case predominante ({} de {} arquivos)",
            kebab,
            snake + camel + kebab
        ))
    };

    if root.join("Cargo.toml").exists() {
        frameworks.push("Rust #[cfg(test)] mod tests inline".to_string());
    }
    if root.join("go.mod").exists() {
        frameworks.push("Go *_test.go com testing package".to_string());
    }
    if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
        if has_file_recursive(root, "pytest.ini") || has_file_recursive(root, "conftest.py") {
            frameworks.push("Python pytest".to_string());
        } else {
            frameworks.push("Python unittest/pytest".to_string());
        }
    }
    if root.join("package.json").exists() {
        if let Ok(pkg) = std::fs::read_to_string(root.join("package.json")) {
            if pkg.contains("\"jest\"") {
                frameworks.push("JavaScript Jest".to_string());
            } else if pkg.contains("\"vitest\"") {
                frameworks.push("JavaScript Vitest".to_string());
            } else if pkg.contains("\"mocha\"") {
                frameworks.push("JavaScript Mocha".to_string());
            }
        }
    }

    ConventionHints {
        naming,
        test_frameworks: frameworks,
    }
}

fn has_file_recursive(root: &Path, name: &str) -> bool {
    WalkDir::new(root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy() == name)
}

fn suggest_commands(stacks: &[StackHint], root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let langs: Vec<&str> = stacks.iter().map(|s| s.language.as_str()).collect();
    let rust_manifest = stacks.iter().find(|s| s.language == "Rust");
    let pkg_manifest = stacks.iter().find(|s| s.manifest.ends_with("package.json"));

    if let Some(m) = rust_manifest {
        let dir = std::path::Path::new(&m.manifest)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty());
        match &dir {
            Some(d) => {
                out.push(format!("cargo build --release (in {})", d));
                out.push(format!("cargo test --workspace (in {})", d));
                out.push(format!(
                    "cargo clippy --all-targets -- -D warnings (in {})",
                    d
                ));
            }
            None => {
                out.push("cargo build --release".to_string());
                out.push("cargo test --workspace".to_string());
                out.push("cargo clippy --all-targets -- -D warnings".to_string());
            }
        }
    }
    if langs.contains(&"Go") {
        out.push("go build ./...".to_string());
        out.push("go test ./...".to_string());
        out.push("go vet ./...".to_string());
    }
    if langs.contains(&"Python") {
        if root.join("pyproject.toml").exists() {
            out.push("pip install -e . (or poetry install)".to_string());
        } else if root.join("requirements.txt").exists() {
            out.push("pip install -r requirements.txt".to_string());
        }
    }
    if let Some(m) = pkg_manifest {
        let abs = root.join(&m.manifest);
        if let Ok(pkg) = std::fs::read_to_string(&abs) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&pkg) {
                if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                    for (k, v) in scripts.iter().take(5) {
                        if let Some(cmd) = v.as_str() {
                            out.push(format!("npm run {} ({})", k, cmd));
                        }
                    }
                }
            }
        }
    }
    if root.join("Makefile").exists() {
        if let Ok(mk) = std::fs::read_to_string(root.join("Makefile")) {
            for line in mk.lines().take(50) {
                if let Some(target) = line.split(':').next() {
                    let target = target.trim();
                    if !target.is_empty()
                        && !target.starts_with('.')
                        && !target.starts_with('#')
                        && target
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                        && out.len() < 15
                    {
                        out.push(format!("make {}", target));
                    }
                }
            }
        }
    }

    out
}

pub fn render_markdown(report: &GlanceReport) -> String {
    let mut s = String::new();
    s.push_str("# Quick context\n\n");
    s.push_str(&format!(
        "Generated by `/fp:quick` in {}ms at {}\n\n",
        report.elapsed_ms, report.generated_at
    ));

    s.push_str("## Stacks\n\n");
    if report.stacks.is_empty() {
        s.push_str("Nenhum manifest reconhecido detectado.\n\n");
    } else {
        for st in &report.stacks {
            s.push_str(&format!("- **{}** ({})\n", st.language, st.manifest));
        }
        s.push('\n');
    }

    s.push_str("## Entry points\n\n");
    if report.entry_points.is_empty() {
        s.push_str("Nenhum entry point classico encontrado nos 5 primeiros niveis.\n\n");
    } else {
        for ep in &report.entry_points {
            s.push_str(&format!("- `{}`\n", ep));
        }
        s.push('\n');
    }

    s.push_str("## Top simbolos (amostra heuristica)\n\n");
    if report.top_symbols.is_empty() {
        s.push_str("Sem simbolos extraidos.\n\n");
    } else {
        s.push_str("| Nome | Kind | Arquivo:linha |\n");
        s.push_str("|------|------|---------------|\n");
        for sym in &report.top_symbols {
            s.push_str(&format!(
                "| `{}` | {} | `{}:{}` |\n",
                sym.name, sym.kind, sym.file, sym.line
            ));
        }
        s.push('\n');
    }

    if let Some(git) = &report.git_activity {
        s.push_str("## Atividade git (90d)\n\n");

        if !git.recent_commits.is_empty() {
            s.push_str("### Ultimos commits\n\n```\n");
            for c in &git.recent_commits {
                s.push_str(c);
                s.push('\n');
            }
            s.push_str("```\n\n");
        }

        if !git.hot_files.is_empty() {
            s.push_str("### Hot files\n\n");
            for h in &git.hot_files {
                s.push_str(&format!("- `{}` ({}x)\n", h.path, h.change_count));
            }
            s.push('\n');
        }

        if !git.active_authors.is_empty() {
            s.push_str("### Autores ativos\n\n");
            for a in &git.active_authors {
                s.push_str(&format!("- {} ({} commits)\n", a.name, a.commit_count));
            }
            s.push('\n');
        }
    }

    s.push_str("## Convencoes detectadas\n\n");
    if let Some(n) = &report.conventions.naming {
        s.push_str(&format!("- Naming: {}\n", n));
    }
    if !report.conventions.test_frameworks.is_empty() {
        s.push_str("- Testes:\n");
        for f in &report.conventions.test_frameworks {
            s.push_str(&format!("  - {}\n", f));
        }
    }
    s.push('\n');

    s.push_str("## Comandos uteis\n\n");
    if report.commands.is_empty() {
        s.push_str("Nao foi possivel inferir comandos pelos manifests.\n\n");
    } else {
        for c in &report.commands {
            s.push_str(&format!("- `{}`\n", c));
        }
        s.push('\n');
    }

    s.push_str("---\n\n");
    s.push_str(
        "Para analise completa (reuse index, reconciliation, co-change graph, provenance):\n\n",
    );
    s.push_str("    /fp:init\n\n");
    s.push_str("Outros comandos:\n\n");
    s.push_str("    /fp:lsp-status     # detecta LSP servers\n");
    s.push_str("    /fp:check          # status do projeto\n");
    s.push_str("    /fp:features       # matriz feature x status\n");

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_stack() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname='x'\n").unwrap();
        let stacks = detect_stacks(tmp.path());
        assert!(stacks.iter().any(|s| s.language == "Rust"));
    }

    #[test]
    fn detects_multiple_stacks() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(tmp.path().join("go.mod"), "module x").unwrap();
        let stacks = detect_stacks(tmp.path());
        assert!(stacks.iter().any(|s| s.language == "Rust"));
        assert!(stacks.iter().any(|s| s.language == "Go"));
    }

    #[test]
    fn samples_top_level_function() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("a.rs"),
            "pub fn hello() {}\npub struct Foo {}\n",
        )
        .unwrap();
        let syms = sample_symbols(tmp.path(), 10);
        assert!(syms.iter().any(|s| s.name == "hello"));
        assert!(syms.iter().any(|s| s.name == "Foo"));
    }

    #[test]
    fn glance_produces_markdown_with_sections() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname='x'\n").unwrap();
        std::fs::write(tmp.path().join("main.rs"), "pub fn main() {}\n").unwrap();
        let report = glance(tmp.path());
        let md = render_markdown(&report);
        assert!(md.contains("# Quick context"));
        assert!(md.contains("## Stacks"));
        assert!(md.contains("Rust"));
        assert!(md.contains("## Comandos uteis"));
    }

    #[test]
    fn suggests_cargo_for_rust_project() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\n").unwrap();
        let stacks = detect_stacks(tmp.path());
        let cmds = suggest_commands(&stacks, tmp.path());
        assert!(cmds.iter().any(|c| c.contains("cargo build")));
        assert!(cmds.iter().any(|c| c.contains("cargo test")));
    }
}
