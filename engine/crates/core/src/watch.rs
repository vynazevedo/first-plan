//! Filesystem watcher with debounced events.
//!
//! Monitors a project tree and emits events when relevant source files change.
//! Events are batched within a debounce window (default 5s for interactive use,
//! 5min for production refresh triggers) so a flurry of edits collapses into a
//! single event.
//!
//! The watcher does NOT regenerate `.first-plan/` automatically - it emits
//! signals on stdout and lets the caller decide what to do (typically pipe into
//! `/first-plan:refresh` or a custom workflow).

use crate::symbols::language_from_path;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// A single debounced event describing one or more file changes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatchEvent {
    pub timestamp: DateTime<Utc>,
    pub affected_paths: Vec<String>,
    pub languages: Vec<String>,
    pub event_count: u32,
}

/// Default paths to ignore (mirrors collect_symbols ignores).
fn default_ignores() -> Vec<&'static str> {
    vec![
        ".git",
        "node_modules",
        "vendor",
        "target",
        "dist",
        "build",
        ".next",
        ".nuxt",
        "venv",
        ".venv",
        "__pycache__",
        ".cache",
        "coverage",
        ".first-plan/cache",
    ]
}

fn should_skip(path: &Path, ignores: &[&str]) -> bool {
    let s = path.to_string_lossy();
    ignores.iter().any(|p| s.contains(p))
}

/// Callback invoked for each debounced event batch.
pub type WatchCallback = Box<dyn FnMut(WatchEvent) + Send>;

/// Run the watcher synchronously, invoking `on_event` for each debounced batch.
///
/// Blocks the current thread until the watcher is dropped or an unrecoverable
/// error occurs. Use a separate thread/process if you need concurrency.
pub fn watch(root: &Path, debounce: Duration, mut on_event: WatchCallback) -> Result<()> {
    let root = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;
    let ignores = default_ignores();
    let (tx, rx) = mpsc::channel::<DebounceEventResult>();

    let mut debouncer = new_debouncer(debounce, move |res| {
        let _ = tx.send(res);
    })
    .context("create debouncer")?;

    debouncer
        .watcher()
        .watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", root.display()))?;

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let mut affected: Vec<String> = Vec::new();
                let mut languages: std::collections::BTreeSet<String> =
                    std::collections::BTreeSet::new();

                for ev in &events {
                    if !matches!(ev.kind, DebouncedEventKind::Any) {
                        continue;
                    }
                    if should_skip(&ev.path, &ignores) {
                        continue;
                    }
                    if let Some(lang) = language_from_path(&ev.path) {
                        languages.insert(lang.to_string());
                    } else {
                        continue;
                    }
                    let rel = ev.path.strip_prefix(&root).unwrap_or(&ev.path);
                    affected.push(rel.to_string_lossy().into_owned());
                }

                if affected.is_empty() {
                    continue;
                }

                affected.sort();
                affected.dedup();

                let event = WatchEvent {
                    timestamp: Utc::now(),
                    affected_paths: affected,
                    languages: languages.into_iter().collect(),
                    event_count: events.len() as u32,
                };
                on_event(event);
            }
            Ok(Err(errs)) => {
                eprintln!("watcher error: {:?}", errs);
            }
            Err(_) => break,
        }
    }

    Ok(())
}

/// Build a WatchEvent for a synthetic list of paths (useful for tests).
pub fn make_event(root: &Path, paths: Vec<PathBuf>) -> WatchEvent {
    let mut affected: Vec<String> = Vec::new();
    let mut languages: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for p in &paths {
        if let Some(lang) = language_from_path(p) {
            languages.insert(lang.to_string());
        }
        let rel = p.strip_prefix(root).unwrap_or(p);
        affected.push(rel.to_string_lossy().into_owned());
    }
    affected.sort();
    affected.dedup();
    WatchEvent {
        timestamp: Utc::now(),
        affected_paths: affected,
        languages: languages.into_iter().collect(),
        event_count: paths.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_event_filters_languages_correctly() {
        let root = PathBuf::from("/tmp/proj");
        let paths = vec![
            root.join("src/main.go"),
            root.join("src/lib.rs"),
            root.join("script.sh"),
            root.join("README.md"),
        ];
        let ev = make_event(&root, paths);
        assert_eq!(ev.affected_paths.len(), 4);
        let langs: std::collections::HashSet<&str> =
            ev.languages.iter().map(|s| s.as_str()).collect();
        assert!(langs.contains("go"));
        assert!(langs.contains("rust"));
        assert!(langs.contains("bash"));
        assert!(!langs.contains("markdown"));
    }

    #[test]
    fn make_event_paths_are_relative() {
        let root = PathBuf::from("/tmp/proj");
        let paths = vec![root.join("a/b.go")];
        let ev = make_event(&root, paths);
        assert_eq!(ev.affected_paths[0], "a/b.go");
    }

    #[test]
    fn should_skip_respects_ignores() {
        let ignores = default_ignores();
        assert!(should_skip(
            Path::new("/tmp/proj/node_modules/x.js"),
            &ignores
        ));
        assert!(should_skip(Path::new("/tmp/proj/.git/HEAD"), &ignores));
        assert!(should_skip(
            Path::new("/tmp/proj/.first-plan/cache/foo"),
            &ignores
        ));
        assert!(!should_skip(Path::new("/tmp/proj/src/main.go"), &ignores));
    }
}
