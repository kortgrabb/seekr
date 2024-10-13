use crate::app::flag::Flags;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::io::{self};
use std::path::{Path, PathBuf};

use super::matcher::search_file_for_patterns;

// Get all files from a list of provided paths
pub fn search_files(
    needle: &str,
    provided: &[String],
    flags: &Flags,
) -> Result<Vec<PathBuf>, io::Error> {
    provided
        .par_iter() // Use parallel iterator
        .map(|path_name| retrieve_files_from_path(needle, path_name, flags))
        .collect::<Result<Vec<_>, io::Error>>() // Collect results into a Result<Vec<PathBuf>, io::Error>
        .map(|files| files.into_iter().flatten().collect()) // Flatten the nested Vecs
}

// Get files from a single path
pub fn retrieve_files_from_path(
    needle: &str,
    path_name: &str,
    flags: &Flags,
) -> Result<Vec<PathBuf>, io::Error> {
    let path = Path::new(path_name);
    if path.is_dir() {
        // Get files from directory
        Ok(retrieve_files_from_dir(needle, path, flags))
    } else if path.is_file() {
        // Call search on file here
        if let Err(e) = search_file_for_patterns(needle, path_name, flags) {
            eprintln!("Error searching file '{}': {}", path_name, e);
        }
        Ok(vec![path.to_path_buf()])
    } else {
        // Return an error if the path is neither a file nor a directory
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{}' is neither a file nor a directory.", path_name),
        ))
    }
}

// Get files from a directory, considering the recursive flag
pub fn retrieve_files_from_dir(needle: &str, path: &Path, flags: &Flags) -> Vec<PathBuf> {
    WalkBuilder::new(path)
        .hidden(!flags.hidden.is_enabled()) // Enable ignoring hidden files if flag is set
        .build()
        .par_bridge() // Use parallel iterator
        .filter_map(std::result::Result::ok) // Filter out any errors
        .filter(|entry| entry.path().is_file()) // Filter out directories
        .map(|entry| {
            let file_str = entry.path().to_string_lossy();
            if let Err(e) = search_file_for_patterns(needle, &file_str, flags) {
                eprintln!("Error searching file '{}': {}", entry.path().display(), e);
            } // Call search on each file and handle errors
            entry.path().to_path_buf()
        })
        .collect()
}
