use crate::app::flag::Flags;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::io::{self};
use std::path::{Path, PathBuf};

use super::matcher::search_single_file;

// Get all files from a list of provided paths
pub fn get_all_files(
    needle: &str,
    provided: &[String],
    flags: &Flags,
) -> Result<Vec<PathBuf>, io::Error> {
    let all_files: Result<Vec<_>, io::Error> = provided
        .par_iter() // Use parallel iterator
        .map(|path_name| get_files_from_path(needle, path_name, flags))
        .collect(); // Collect results into a Result<Vec<PathBuf>, io::Error>

    all_files.map(|files| files.into_iter().flatten().collect()) // Flatten the nested Vecs
}

// Get files from a single path
pub fn get_files_from_path(
    needle: &str,
    path_name: &str,
    flags: &Flags,
) -> Result<Vec<PathBuf>, io::Error> {
    let path = Path::new(path_name);
    if path.is_dir() {
        // Get files from directory
        Ok(get_files_from_directory(needle, path, flags))
    } else if path.is_file() {
        // call search on file here
        if let Err(e) = search_single_file(needle, path_name, flags) {
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
pub fn get_files_from_directory(needle: &str, path: &Path, flags: &Flags) -> Vec<PathBuf> {
    // Use WalkBuilder to traverse the directory
    let builder = WalkBuilder::new(path)
        .hidden(!flags.hidden.is_enabled()) // Enable ignoring hidden files if flag is set
        .build();

    let files: Vec<_> = builder
        .par_bridge() // Use parallel iterator
        .filter_map(std::result::Result::ok) // Filter out any errors
        .filter(|entry| entry.path().is_file()) // Filter out directories
        .map(|entry| {
            if let Err(e) = search_single_file(needle, entry.path().to_str().unwrap(), flags) {
                eprintln!("Error searching file '{}': {}", entry.path().display(), e);
            } // Call search on each file and handle errors
            entry.path().to_path_buf()
        })
        .collect();
    files
}
