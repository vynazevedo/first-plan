//! Parallel xxh3 file hashing for cache invalidation.
//!
//! xxh3 chosen for speed (5-10x faster than SHA-256). Not cryptographic, but
//! cache invalidation does not need cryptographic guarantees.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileHash {
    pub hash: String,
    pub size_bytes: u64,
    pub modified: Option<DateTime<Utc>>,
}

/// Hash multiple files in parallel using rayon.
///
/// Output is a sorted (BTreeMap) map keyed by relative path so the JSON output
/// is deterministic.
pub fn hash_files_parallel(paths: &[PathBuf]) -> Result<BTreeMap<String, FileHash>> {
    let results: Vec<(String, Result<FileHash>)> = paths
        .par_iter()
        .map(|p| {
            let key = p.to_string_lossy().into_owned();
            (key, hash_one(p))
        })
        .collect();

    let mut map = BTreeMap::new();
    for (key, res) in results {
        match res {
            Ok(h) => {
                map.insert(key, h);
            }
            Err(e) => {
                // Skip files that fail to read (deleted, permissions, etc).
                // For cache use case it is fine to omit.
                eprintln!("warning: skipping {}: {}", key, e);
            }
        }
    }
    Ok(map)
}

fn hash_one(path: &Path) -> Result<FileHash> {
    let data = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let hash = xxh3_64(&data);
    let metadata =
        std::fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
        })
        .and_then(|secs| DateTime::<Utc>::from_timestamp(secs, 0));

    Ok(FileHash {
        hash: format!("{:016x}", hash),
        size_bytes: data.len() as u64,
        modified,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn hashes_a_single_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();
        let path = file.path().to_path_buf();

        let result = hash_files_parallel(std::slice::from_ref(&path)).unwrap();
        let entry = result.get(path.to_string_lossy().as_ref()).unwrap();
        assert_eq!(entry.size_bytes, 11);
        assert!(!entry.hash.is_empty());
    }

    #[test]
    fn hashes_multiple_files_in_parallel() {
        let files: Vec<NamedTempFile> = (0..50)
            .map(|i| {
                let mut f = NamedTempFile::new().unwrap();
                writeln!(f, "content {}", i).unwrap();
                f
            })
            .collect();
        let paths: Vec<PathBuf> = files.iter().map(|f| f.path().to_path_buf()).collect();

        let result = hash_files_parallel(&paths).unwrap();
        assert_eq!(result.len(), 50);
    }

    #[test]
    fn deterministic_hash_for_same_content() {
        let mut f1 = NamedTempFile::new().unwrap();
        let mut f2 = NamedTempFile::new().unwrap();
        write!(f1, "same content").unwrap();
        write!(f2, "same content").unwrap();

        let h1 = hash_one(f1.path()).unwrap().hash;
        let h2 = hash_one(f2.path()).unwrap().hash;
        assert_eq!(h1, h2);
    }
}
