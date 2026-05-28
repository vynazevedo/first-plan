//! Coverage report parsing.
//!
//! Suporta formatos comuns:
//! - lcov (`lcov.info`, `coverage/lcov.info`) - Node, Rust (via grcov), C/C++
//! - cobertura XML (`coverage.xml`, `cobertura.xml`) - Python, Java, JS
//! - jacoco XML (`jacoco.xml`, `target/site/jacoco/jacoco.xml`) - Java/Kotlin
//! - jest coverage-summary.json (`coverage/coverage-summary.json`) - Node/TS
//! - go test coverprofile (`coverage.out`) - Go
//!
//! Output: lista de arquivos com % coverage e linhas nao cobertas.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoverageReport {
    pub formats_detected: Vec<String>,
    pub source_files: Vec<FileCoverage>,
    pub overall: OverallStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub path: String,
    pub lines_total: u32,
    pub lines_covered: u32,
    pub percent: f32,
    pub uncovered_ranges: Vec<UncoveredRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverallStats {
    pub files_count: u32,
    pub lines_total: u32,
    pub lines_covered: u32,
    pub percent: f32,
}

pub fn detect(root: &Path) -> CoverageReport {
    let mut report = CoverageReport::default();
    let mut file_map: HashMap<String, FileCoverage> = HashMap::new();

    let candidates = find_coverage_files(root);

    for (format, path) in candidates {
        let parsed = match format.as_str() {
            "lcov" => parse_lcov(&path).ok(),
            "cobertura" => parse_cobertura(&path, root).ok(),
            "jacoco" => parse_jacoco(&path, root).ok(),
            "jest-summary" => parse_jest_summary(&path).ok(),
            "go-coverprofile" => parse_go_coverprofile(&path).ok(),
            _ => None,
        };

        let Some(files) = parsed else {
            continue;
        };

        if !report.formats_detected.contains(&format) {
            report.formats_detected.push(format.clone());
        }

        for f in files {
            file_map
                .entry(f.path.clone())
                .and_modify(|existing| {
                    if f.percent > existing.percent {
                        *existing = f.clone();
                    }
                })
                .or_insert(f);
        }
    }

    let mut files: Vec<FileCoverage> = file_map.into_values().collect();
    files.sort_by(|a, b| {
        a.percent
            .partial_cmp(&b.percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_lines: u32 = files.iter().map(|f| f.lines_total).sum();
    let covered_lines: u32 = files.iter().map(|f| f.lines_covered).sum();
    let percent = if total_lines > 0 {
        (covered_lines as f32 / total_lines as f32) * 100.0
    } else {
        0.0
    };

    report.overall = OverallStats {
        files_count: files.len() as u32,
        lines_total: total_lines,
        lines_covered: covered_lines,
        percent,
    };
    report.source_files = files;
    report
}

fn find_coverage_files(root: &Path) -> Vec<(String, PathBuf)> {
    let candidates: Vec<(String, &str)> = vec![
        ("lcov".into(), "lcov.info"),
        ("lcov".into(), "coverage/lcov.info"),
        ("lcov".into(), "coverage/lcov-report/lcov.info"),
        ("cobertura".into(), "coverage.xml"),
        ("cobertura".into(), "cobertura.xml"),
        ("cobertura".into(), "coverage/cobertura-coverage.xml"),
        ("jacoco".into(), "jacoco.xml"),
        ("jacoco".into(), "target/site/jacoco/jacoco.xml"),
        (
            "jacoco".into(),
            "build/reports/jacoco/test/jacocoTestReport.xml",
        ),
        ("jest-summary".into(), "coverage/coverage-summary.json"),
        ("go-coverprofile".into(), "coverage.out"),
        ("go-coverprofile".into(), "cover.out"),
    ];

    let mut out = Vec::new();
    for (fmt, rel) in candidates {
        let p = root.join(rel);
        if p.exists() && p.is_file() {
            out.push((fmt, p));
        }
    }
    out
}

fn parse_lcov(path: &Path) -> anyhow::Result<Vec<FileCoverage>> {
    let content = std::fs::read_to_string(path)?;
    let mut files = Vec::new();
    let mut current_path: Option<String> = None;
    let mut lines_total: u32 = 0;
    let mut lines_covered: u32 = 0;
    let mut uncovered: Vec<u32> = Vec::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("SF:") {
            current_path = Some(rest.trim().to_string());
            lines_total = 0;
            lines_covered = 0;
            uncovered.clear();
        } else if let Some(rest) = line.strip_prefix("DA:") {
            let mut parts = rest.split(',');
            let line_no: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let hits: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            lines_total += 1;
            if hits > 0 {
                lines_covered += 1;
            } else {
                uncovered.push(line_no);
            }
        } else if line.starts_with("end_of_record") {
            if let Some(p) = current_path.take() {
                let percent = if lines_total > 0 {
                    (lines_covered as f32 / lines_total as f32) * 100.0
                } else {
                    0.0
                };
                files.push(FileCoverage {
                    path: p,
                    lines_total,
                    lines_covered,
                    percent,
                    uncovered_ranges: collapse_ranges(&uncovered),
                });
                lines_total = 0;
                lines_covered = 0;
                uncovered.clear();
            }
        }
    }

    Ok(files)
}

fn parse_cobertura(path: &Path, root: &Path) -> anyhow::Result<Vec<FileCoverage>> {
    let content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut files = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_uncovered: Vec<u32> = Vec::new();
    let mut current_total: u32 = 0;
    let mut current_covered: u32 = 0;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let tag = std::str::from_utf8(name.as_ref()).unwrap_or("");
                if tag == "class" {
                    for attr in e.attributes().with_checks(false).flatten() {
                        if attr.key.as_ref() == b"filename" {
                            let val = String::from_utf8_lossy(&attr.value).into_owned();
                            let _ = root;
                            current_path = Some(val);
                            current_uncovered.clear();
                            current_total = 0;
                            current_covered = 0;
                        }
                    }
                } else if tag == "line" {
                    let mut line_no: u32 = 0;
                    let mut hits: u32 = 0;
                    for attr in e.attributes().with_checks(false).flatten() {
                        match attr.key.as_ref() {
                            b"number" => {
                                line_no = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0)
                            }
                            b"hits" => {
                                hits = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0)
                            }
                            _ => {}
                        }
                    }
                    current_total += 1;
                    if hits > 0 {
                        current_covered += 1;
                    } else if line_no > 0 {
                        current_uncovered.push(line_no);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let tag = std::str::from_utf8(name.as_ref()).unwrap_or("");
                if tag == "line" {
                    let mut line_no: u32 = 0;
                    let mut hits: u32 = 0;
                    for attr in e.attributes().with_checks(false).flatten() {
                        match attr.key.as_ref() {
                            b"number" => {
                                line_no = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0)
                            }
                            b"hits" => {
                                hits = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0)
                            }
                            _ => {}
                        }
                    }
                    current_total += 1;
                    if hits > 0 {
                        current_covered += 1;
                    } else if line_no > 0 {
                        current_uncovered.push(line_no);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let tag = std::str::from_utf8(name.as_ref()).unwrap_or("");
                if tag == "class" {
                    if let Some(p) = current_path.take() {
                        let percent = if current_total > 0 {
                            (current_covered as f32 / current_total as f32) * 100.0
                        } else {
                            0.0
                        };
                        files.push(FileCoverage {
                            path: p,
                            lines_total: current_total,
                            lines_covered: current_covered,
                            percent,
                            uncovered_ranges: collapse_ranges(&current_uncovered),
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(files)
}

fn parse_jacoco(path: &Path, root: &Path) -> anyhow::Result<Vec<FileCoverage>> {
    parse_cobertura(path, root)
}

fn parse_jest_summary(path: &Path) -> anyhow::Result<Vec<FileCoverage>> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let Some(obj) = json.as_object() else {
        return Ok(Vec::new());
    };

    let mut files = Vec::new();
    for (key, value) in obj {
        if key == "total" {
            continue;
        }
        let lines = value.get("lines").and_then(|l| l.as_object());
        let total = lines
            .and_then(|l| l.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let covered = lines
            .and_then(|l| l.get("covered"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let percent = lines
            .and_then(|l| l.get("pct"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        files.push(FileCoverage {
            path: key.clone(),
            lines_total: total,
            lines_covered: covered,
            percent,
            uncovered_ranges: Vec::new(),
        });
    }

    Ok(files)
}

fn parse_go_coverprofile(path: &Path) -> anyhow::Result<Vec<FileCoverage>> {
    let content = std::fs::read_to_string(path)?;
    let mut file_stats: HashMap<String, (u32, u32, Vec<u32>)> = HashMap::new();

    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let location = parts[0];
        let count: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        let (file, range) = match location.rsplit_once(':') {
            Some((f, r)) => (f, r),
            None => continue,
        };

        let (start_part, _) = match range.split_once(',') {
            Some(p) => p,
            None => continue,
        };
        let line_no: u32 = start_part
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let entry = file_stats
            .entry(file.to_string())
            .or_insert((0, 0, Vec::new()));
        entry.0 += 1;
        if count > 0 {
            entry.1 += 1;
        } else if line_no > 0 {
            entry.2.push(line_no);
        }
    }

    let mut files = Vec::new();
    for (path, (total, covered, uncovered)) in file_stats {
        let percent = if total > 0 {
            (covered as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        files.push(FileCoverage {
            path,
            lines_total: total,
            lines_covered: covered,
            percent,
            uncovered_ranges: collapse_ranges(&uncovered),
        });
    }

    Ok(files)
}

fn collapse_ranges(lines: &[u32]) -> Vec<UncoveredRange> {
    if lines.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<u32> = lines.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let mut out = Vec::new();
    let mut start = sorted[0];
    let mut end = sorted[0];
    for &n in sorted.iter().skip(1) {
        if n == end + 1 {
            end = n;
        } else {
            out.push(UncoveredRange { start, end });
            start = n;
            end = n;
        }
    }
    out.push(UncoveredRange { start, end });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_lcov() {
        let tmp = tempfile::tempdir().unwrap();
        let lcov = tmp.path().join("lcov.info");
        fs::write(
            &lcov,
            "TN:\nSF:src/foo.rs\nDA:1,1\nDA:2,0\nDA:3,1\nDA:4,0\nDA:5,0\nend_of_record\n",
        )
        .unwrap();

        let files = parse_lcov(&lcov).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/foo.rs");
        assert_eq!(files[0].lines_total, 5);
        assert_eq!(files[0].lines_covered, 2);
        assert!((files[0].percent - 40.0).abs() < 0.1);
        assert_eq!(files[0].uncovered_ranges.len(), 2);
    }

    #[test]
    fn collapses_consecutive_lines() {
        let ranges = collapse_ranges(&[1, 2, 3, 7, 8, 15]);
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0].start, 1);
        assert_eq!(ranges[0].end, 3);
        assert_eq!(ranges[1].start, 7);
        assert_eq!(ranges[1].end, 8);
        assert_eq!(ranges[2].start, 15);
        assert_eq!(ranges[2].end, 15);
    }

    #[test]
    fn detect_finds_lcov_in_root() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("lcov.info"),
            "SF:src/x.rs\nDA:1,1\nDA:2,1\nend_of_record\n",
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(report.formats_detected.contains(&"lcov".to_string()));
        assert_eq!(report.source_files.len(), 1);
        assert!((report.overall.percent - 100.0).abs() < 0.1);
    }

    #[test]
    fn parses_jest_summary() {
        let tmp = tempfile::tempdir().unwrap();
        let cov = tmp.path().join("coverage/coverage-summary.json");
        fs::create_dir_all(cov.parent().unwrap()).unwrap();
        fs::write(
            &cov,
            r#"{
  "total": {"lines": {"total": 100, "covered": 80, "pct": 80}},
  "/src/app.ts": {"lines": {"total": 50, "covered": 45, "pct": 90}},
  "/src/util.ts": {"lines": {"total": 50, "covered": 35, "pct": 70}}
}"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(report
            .formats_detected
            .contains(&"jest-summary".to_string()));
        assert_eq!(report.source_files.len(), 2);
    }

    #[test]
    fn parses_go_coverprofile() {
        let tmp = tempfile::tempdir().unwrap();
        let prof = tmp.path().join("coverage.out");
        fs::write(
            &prof,
            "mode: set\nexample.com/x/foo.go:1.1,3.2 1 1\nexample.com/x/foo.go:5.1,7.2 1 0\nexample.com/x/bar.go:1.1,2.5 1 1\n",
        )
        .unwrap();

        let files = parse_go_coverprofile(&prof).unwrap();
        assert!(files.iter().any(|f| f.path == "example.com/x/foo.go"));
        let foo = files
            .iter()
            .find(|f| f.path == "example.com/x/foo.go")
            .unwrap();
        assert_eq!(foo.lines_total, 2);
        assert_eq!(foo.lines_covered, 1);
    }

    #[test]
    fn empty_report_when_no_coverage() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.source_files.len(), 0);
        assert_eq!(report.formats_detected.len(), 0);
    }
}
