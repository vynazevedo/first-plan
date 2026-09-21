//! Load and aggregate IR from `.first-plan/` into structured context for adapters.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IrContext {
    pub root: String,
    pub project_name: String,
    pub has_ir: bool,
    pub sections: Vec<IrSection>,
    pub quick_glance: Option<String>,
    pub stacks: Vec<String>,
    pub key_conventions: Vec<String>,
    pub reuse_summary: Option<String>,
    pub features_summary: Option<String>,
    pub quality_summary: Option<String>,
    pub contracts_summary: Option<String>,
    pub evolution_summary: Option<String>,
    pub runtime_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrSection {
    pub number: String,
    pub name: String,
    pub file: String,
    pub content: String,
    pub excerpt: String,
}

pub fn load_ir(root: &Path) -> Result<IrContext> {
    let mut ctx = IrContext {
        root: root.to_string_lossy().into_owned(),
        project_name: root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string(),
        ..Default::default()
    };

    let ir_dir = root.join(".first-plan");
    if !ir_dir.exists() {
        return Ok(ctx);
    }
    ctx.has_ir = true;

    // Try to load quick glance if exists
    let quick_path = ir_dir.join("quick").join("00-glance.md");
    if quick_path.exists() {
        ctx.quick_glance = std::fs::read_to_string(&quick_path).ok();
    }

    // Scan .first-plan/ for numbered directories
    if let Ok(rd) = std::fs::read_dir(&ir_dir) {
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_ir_section(&name) {
                continue;
            }
            if let Some(section) = load_section(&entry.path(), &name) {
                ctx.sections.push(section);
            }
        }
    }

    // Extract summaries per layer
    for section in &ctx.sections {
        match section.number.as_str() {
            "00" | "01" => ctx.stacks.push(section.excerpt.clone()),
            "02" => ctx.key_conventions.push(section.excerpt.clone()),
            "03" => ctx.reuse_summary = Some(section.excerpt.clone()),
            "09" => ctx.features_summary = Some(section.excerpt.clone()),
            "11" => ctx.quality_summary = Some(section.excerpt.clone()),
            "12" => ctx.contracts_summary = Some(section.excerpt.clone()),
            "13" => ctx.evolution_summary = Some(section.excerpt.clone()),
            "14" => ctx.runtime_summary = Some(section.excerpt.clone()),
            _ => {}
        }
    }

    Ok(ctx)
}

fn is_ir_section(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() && bytes[2] == b'-'
}

fn load_section(path: &Path, dir_name: &str) -> Option<IrSection> {
    let number = dir_name[..2].to_string();
    let name = dir_name[3..].to_string();
    let mut files: Vec<_> = walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.into_path())
        .collect();
    files.sort();
    let mut content = String::new();
    let mut excerpt = String::new();
    for file in files {
        let text = std::fs::read_to_string(&file).ok()?;
        let relative = file.strip_prefix(path).ok()?.to_string_lossy();
        let header = format!("\n### .first-plan/{}/{}\n", dir_name, relative);
        content.push_str(&header);
        content.push_str(&text);
        excerpt.push_str(&header);
        excerpt.push_str(&extract_excerpt(&text, 800));
    }
    if content.is_empty() {
        return None;
    }
    Some(IrSection {
        number,
        name,
        file: format!(".first-plan/{}", dir_name),
        content,
        excerpt,
    })
}

fn extract_excerpt(content: &str, max_chars: usize) -> String {
    let body = content
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or(content);
    let cleaned = body
        .lines()
        .filter(|line| !line.trim_start().starts_with("<!--"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut excerpt: String = cleaned.chars().take(max_chars).collect();
    if cleaned.chars().count() > max_chars {
        excerpt.push_str("\n...");
    }
    excerpt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_ir_section_dirs() {
        assert!(is_ir_section("00-discovery"));
        assert!(is_ir_section("11-quality"));
        assert!(!is_ir_section("quick"));
        assert!(!is_ir_section("cache"));
        assert!(!is_ir_section("meta"));
    }

    #[test]
    fn returns_empty_ctx_when_no_ir() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = load_ir(tmp.path()).unwrap();
        assert!(!ctx.has_ir);
    }

    #[test]
    fn loads_ir_with_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let ir_dir = tmp.path().join(".first-plan");
        std::fs::create_dir(&ir_dir).unwrap();
        let sec_dir = ir_dir.join("02-conventions");
        std::fs::create_dir(&sec_dir).unwrap();
        std::fs::write(
            sec_dir.join("naming.md"),
            "# Naming\n\nsnake_case for files\n",
        )
        .unwrap();

        let ctx = load_ir(tmp.path()).unwrap();
        assert!(ctx.has_ir);
        assert_eq!(ctx.sections.len(), 1);
        assert_eq!(ctx.sections[0].number, "02");
        assert_eq!(ctx.sections[0].name, "conventions");
    }

    #[test]
    fn includes_every_convention_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".first-plan/02-conventions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("naming.md"), "naming rule").unwrap();
        std::fs::write(dir.join("testing.md"), "testing rule").unwrap();
        let context = load_ir(tmp.path()).unwrap();
        assert!(context.key_conventions[0].contains("testing rule"));
        assert!(context.key_conventions[0].contains("naming rule"));
        assert!(!is_ir_section("é-example"));
    }
    #[test]
    fn extract_excerpt_respects_max() {
        let long = "line\n".repeat(1000);
        let ex = extract_excerpt(&long, 100);
        assert!(ex.len() < 200);
        assert!(ex.ends_with("...") || ex.len() < 200);
    }
}
