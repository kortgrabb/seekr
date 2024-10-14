use crate::app::flags::Flags;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::matcher::search_file_for_patterns;

pub struct SearchResult {
    pub has_match: bool,
    // You can extend this struct with additional fields like `stats` or `errors` if needed.
}

impl SearchResult {
    /// Returns true if there was at least one match.
    pub fn has_match(&self) -> bool {
        self.has_match
    }
}

/// Function to search files in a single-threaded manner
pub fn search_files(
    needle: &str,
    files: &[String],
    flags: &Flags,
    matched: &AtomicBool,
) -> Result<SearchResult, io::Error> {
    let mut has_any_match = false;

    for file in files {
        let path = Path::new(file);

        if path.is_dir() {
            for entry in WalkBuilder::new(path)
                .hidden(!flags.hidden.is_enabled())
                .build()
                .filter_map(Result::ok)
            {
                if entry.path().is_file() {
                    if let Ok(has_match) = search_file(needle, entry.path(), flags) {
                        if has_match {
                            matched.store(true, Ordering::SeqCst);
                            has_any_match = true;
                        }
                    }
                }
            }
        } else if path.is_file() {
            if let Ok(has_match) = search_file(needle, path, flags) {
                if has_match {
                    matched.store(true, Ordering::SeqCst);
                    has_any_match = true;
                }
            }
        }
    }

    Ok(SearchResult {
        has_match: has_any_match,
    })
}

/// Function to search files in parallel
pub fn search_files_parallel(
    needle: &str,
    files: &[String],
    flags: &Flags,
    matched: &AtomicBool,
) -> Result<SearchResult, io::Error> {
    let has_any_match = AtomicBool::new(false);

    files.par_iter().for_each(|file| {
        let path = Path::new(file);

        if path.is_dir() {
            WalkBuilder::new(path)
                .hidden(!flags.hidden.is_enabled())
                .build()
                .par_bridge()
                .filter_map(Result::ok)
                .for_each(|entry| {
                    if entry.path().is_file() {
                        if let Ok(has_match) = search_file(needle, entry.path(), flags) {
                            if has_match {
                                matched.store(true, Ordering::SeqCst);
                                has_any_match.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                });
        } else if path.is_file() {
            if let Ok(has_match) = search_file(needle, path, flags) {
                if has_match {
                    matched.store(true, Ordering::SeqCst);
                    has_any_match.store(true, Ordering::SeqCst);
                }
            }
        }
    });

    Ok(SearchResult {
        has_match: has_any_match.load(Ordering::SeqCst),
    })
}

/// Helper function to search within a file
fn search_file(needle: &str, path: &Path, flags: &Flags) -> Result<bool, io::Error> {
    let file_path = path.to_string_lossy();
    search_file_for_patterns(needle, &file_path, flags)
}
