//! Shared helpers for the integration tests.
//!
//! This module is compiled separately into every test binary, so a helper only
//! one of them needs still counts as dead code in the others.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// A scratch directory that removes itself when the test ends.
pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "opencollection-{label}-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("scratch directory should be creatable");
        TempDir(path)
    }

    pub fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Recursively copy `from` into `to`.
///
/// Tests that save into a fixture must work on a throwaway copy — saving now
/// prunes, and a bug would edit the fixture in the repository.
pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("target directory should be creatable");
    for entry in std::fs::read_dir(from).expect("source directory should be readable") {
        let entry = entry.expect("entry should be readable");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("file should be copyable");
        }
    }
}
