use crate::app::flag::Flags;
use ignore::WalkBuilder;
use std::io;
use std::path::{Path, PathBuf};

// Get all files from a list of provided paths
pub fn get_all_files(provided: &[String], flags: &Flags) -> Result<Vec<PathBuf>, io::Error> {
    let mut all_files = Vec::new();
    // Iterate through each provided path and collect all files
    for path_name in provided {
        all_files.extend(get_files_from_path(path_name, flags)?);
    }

    Ok(all_files)
}

// Get files from a specific path, handling directories and files
pub fn get_files_from_path(path_name: &str, flags: &Flags) -> Result<Vec<PathBuf>, io::Error> {
    let path = Path::new(path_name);
    if path.is_dir() {
        // If the path is a directory, get all files from the directory
        get_files_from_directory(path, flags)
    } else if path.is_file() {
        // If the path is a file, return it as a PathBuf
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
pub fn get_files_from_directory(path: &Path, flags: &Flags) -> Result<Vec<PathBuf>, io::Error> {
    // Use WalkBuilder to traverse the directory
    let mut builder = WalkBuilder::new(path);
    if !flags.recursive {
        builder.max_depth(Some(1)); // Limit the depth to 1 if not recursive
    }

    let files: Vec<_> = builder
        .hidden(!flags.hidden) // Enable ignoring hidden files if flag is set
        .build()
        .filter_map(|entry| entry.ok()) // Filter out any errors
        .filter(|entry| entry.path().is_file()) // Filter out directories
        .map(|entry| entry.path().to_path_buf()) // Convert to PathBufs
        .collect();

    Ok(files)
}
