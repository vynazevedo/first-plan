//! Co-change graph analysis.
//!
//! Given a set of commits with their changed files, builds a matrix of
//! "files A and B were modified together in N commits" pairs, computes
//! ratios, classifies strength and detects clusters via Union-Find.

use crate::git::Commit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filters applied during matrix construction.
#[derive(Debug, Clone)]
pub struct Filters {
    pub min_occurrences: u32,
    pub min_ratio: f32,
    pub exclude_patterns: Vec<String>,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            min_occurrences: 5,
            min_ratio: 0.5,
            exclude_patterns: default_excludes(),
        }
    }
}

fn default_excludes() -> Vec<String> {
    vec![
        "*.lock".into(),
        "*.lockb".into(),
        "package-lock.json".into(),
        "yarn.lock".into(),
        "Cargo.lock".into(),
        "go.sum".into(),
        "composer.lock".into(),
        "Gemfile.lock".into(),
        "*.min.js".into(),
        "*.bundle.js".into(),
        "vendor/".into(),
        "node_modules/".into(),
        "target/".into(),
        ".gen.".into(),
        ".pb.".into(),
        "_pb2.py".into(),
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Strength {
    Strong,
    Moderate,
    Weak,
}

impl Strength {
    fn from_ratio(ratio: f32) -> Self {
        if ratio >= 0.9 {
            Self::Strong
        } else if ratio >= 0.7 {
            Self::Moderate
        } else {
            Self::Weak
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pair {
    pub file_a: String,
    pub file_b: String,
    pub co_change_ratio: f32,
    pub shared_commits: u32,
    pub total_a: u32,
    pub total_b: u32,
    pub strength: Strength,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cluster {
    pub id: String,
    pub files: Vec<String>,
    pub internal_cohesion: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoChangeMatrix {
    pub pairs: Vec<Pair>,
    pub clusters: Vec<Cluster>,
    pub total_files_analyzed: u32,
}

/// Build the co-change matrix from a set of commits with their changed files.
pub fn build_matrix(commits: &[Commit], filters: &Filters) -> CoChangeMatrix {
    // Count individual file occurrences
    let mut file_count: HashMap<&str, u32> = HashMap::new();
    for c in commits {
        for f in &c.files {
            if !is_excluded(f, &filters.exclude_patterns) {
                *file_count.entry(f.as_str()).or_insert(0) += 1;
            }
        }
    }

    // Count co-occurrences for files that pass the min_occurrences threshold
    let mut pair_count: HashMap<(String, String), u32> = HashMap::new();
    for c in commits {
        let kept: Vec<&String> = c
            .files
            .iter()
            .filter(|f| {
                !is_excluded(f, &filters.exclude_patterns)
                    && file_count.get(f.as_str()).copied().unwrap_or(0) >= filters.min_occurrences
            })
            .collect();
        for i in 0..kept.len() {
            for j in (i + 1)..kept.len() {
                let (a, b) = if kept[i] < kept[j] {
                    (kept[i].clone(), kept[j].clone())
                } else {
                    (kept[j].clone(), kept[i].clone())
                };
                *pair_count.entry((a, b)).or_insert(0) += 1;
            }
        }
    }

    // Compute ratios and filter
    let mut pairs: Vec<Pair> = pair_count
        .into_iter()
        .filter_map(|((a, b), shared)| {
            let total_a = file_count.get(a.as_str()).copied().unwrap_or(0);
            let total_b = file_count.get(b.as_str()).copied().unwrap_or(0);
            let denom = total_a.max(total_b) as f32;
            if denom == 0.0 {
                return None;
            }
            let ratio = shared as f32 / denom;
            if ratio < filters.min_ratio {
                return None;
            }
            Some(Pair {
                file_a: a,
                file_b: b,
                co_change_ratio: ratio,
                shared_commits: shared,
                total_a,
                total_b,
                strength: Strength::from_ratio(ratio),
            })
        })
        .collect();

    // Sort by ratio desc, then by shared_commits desc for stable output
    pairs.sort_by(|a, b| {
        b.co_change_ratio
            .partial_cmp(&a.co_change_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.shared_commits.cmp(&a.shared_commits))
            .then(a.file_a.cmp(&b.file_a))
    });

    let clusters = detect_clusters(&pairs, 0.7);
    let total_files_analyzed = file_count.len() as u32;

    CoChangeMatrix {
        pairs,
        clusters,
        total_files_analyzed,
    }
}

fn is_excluded(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| matches_pattern(path, p))
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('/') {
        return path.contains(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return path.ends_with(suffix);
    }
    path.contains(pattern)
}

/// Union-Find based cluster detection on pairs above a strength threshold.
pub fn detect_clusters(pairs: &[Pair], threshold: f32) -> Vec<Cluster> {
    let mut parent: HashMap<String, String> = HashMap::new();

    fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
        let p = parent.get(x).cloned().unwrap_or_else(|| x.to_string());
        if p == x {
            parent.entry(x.to_string()).or_insert_with(|| x.to_string());
            return x.to_string();
        }
        let root = find(parent, &p);
        parent.insert(x.to_string(), root.clone());
        root
    }

    fn union(parent: &mut HashMap<String, String>, a: &str, b: &str) {
        let root_a = find(parent, a);
        let root_b = find(parent, b);
        if root_a != root_b {
            parent.insert(root_a, root_b);
        }
    }

    // Build clusters from pairs above threshold
    for p in pairs.iter().filter(|p| p.co_change_ratio >= threshold) {
        union(&mut parent, &p.file_a, &p.file_b);
    }

    // Group nodes by root
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    let nodes: Vec<String> = parent.keys().cloned().collect();
    for node in &nodes {
        let root = find(&mut parent, node);
        groups.entry(root).or_default().push(node.clone());
    }

    // Build clusters with cohesion (avg ratio of internal pairs)
    let mut clusters: Vec<Cluster> = groups
        .into_iter()
        .filter(|(_, files)| files.len() >= 2)
        .enumerate()
        .map(|(idx, (_root, mut files))| {
            files.sort();
            let cohesion = compute_cohesion(&files, pairs);
            Cluster {
                id: format!("cluster-{}", idx + 1),
                files,
                internal_cohesion: cohesion,
            }
        })
        .collect();

    clusters.sort_by(|a, b| b.files.len().cmp(&a.files.len()));
    clusters
}

fn compute_cohesion(files: &[String], pairs: &[Pair]) -> f32 {
    let file_set: std::collections::HashSet<&str> = files.iter().map(|s| s.as_str()).collect();
    let internal: Vec<&Pair> = pairs
        .iter()
        .filter(|p| file_set.contains(p.file_a.as_str()) && file_set.contains(p.file_b.as_str()))
        .collect();
    if internal.is_empty() {
        return 0.0;
    }
    let sum: f32 = internal.iter().map(|p| p.co_change_ratio).sum();
    sum / internal.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(sha: &str, files: &[&str]) -> Commit {
        Commit {
            sha: sha.into(),
            files: files.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn basic_pair_detection() {
        let commits: Vec<Commit> = (0..6)
            .map(|i| commit(&format!("c{}", i), &["a.go", "b.go"]))
            .collect();

        let filters = Filters {
            min_occurrences: 5,
            min_ratio: 0.5,
            exclude_patterns: vec![],
        };

        let m = build_matrix(&commits, &filters);
        assert_eq!(m.pairs.len(), 1);
        assert_eq!(m.pairs[0].file_a, "a.go");
        assert_eq!(m.pairs[0].file_b, "b.go");
        assert_eq!(m.pairs[0].co_change_ratio, 1.0);
        assert_eq!(m.pairs[0].shared_commits, 6);
        assert_eq!(m.pairs[0].strength, Strength::Strong);
    }

    #[test]
    fn excludes_below_min_occurrences() {
        let commits = vec![commit("c1", &["a.go", "b.go"])];
        let filters = Filters {
            min_occurrences: 5,
            min_ratio: 0.5,
            exclude_patterns: vec![],
        };
        let m = build_matrix(&commits, &filters);
        assert!(m.pairs.is_empty());
    }

    #[test]
    fn respects_exclude_patterns() {
        let commits: Vec<Commit> = (0..10)
            .map(|i| commit(&format!("c{}", i), &["a.go", "package-lock.json"]))
            .collect();

        let m = build_matrix(&commits, &Filters::default());
        assert!(m.pairs.is_empty(), "lockfile should be excluded");
    }

    #[test]
    fn detects_strength_levels() {
        // Pair shared 9/10 commits => ratio 0.9 strong
        let mut commits: Vec<Commit> = (0..9)
            .map(|i| commit(&format!("c{}", i), &["a.go", "b.go"]))
            .collect();
        commits.push(commit("c9", &["a.go"]));

        let m = build_matrix(
            &commits,
            &Filters {
                min_occurrences: 5,
                min_ratio: 0.5,
                exclude_patterns: vec![],
            },
        );
        assert_eq!(m.pairs[0].strength, Strength::Strong);
    }

    #[test]
    fn detects_cluster() {
        let commits: Vec<Commit> = (0..10)
            .map(|i| commit(&format!("c{}", i), &["a.go", "b.go", "c.go"]))
            .collect();

        let m = build_matrix(
            &commits,
            &Filters {
                min_occurrences: 5,
                min_ratio: 0.5,
                exclude_patterns: vec![],
            },
        );
        assert_eq!(m.clusters.len(), 1);
        assert_eq!(m.clusters[0].files.len(), 3);
        assert!(m.clusters[0].internal_cohesion >= 0.99);
    }
}
