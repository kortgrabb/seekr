use crate::app::flag::Flags;
use ignore::WalkBuilder;
use std::io::{self};
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
        Ok(get_files_from_directory(path, flags))
    } else if path.is_file() {
        // If the path is a file, return it as a PathBuf
        Ok(vec![path.to_path_buf()])
    } else {
        // Return an error if the path is neither a file nor a directory
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{path_name}' is neither a file nor a directory."),
        ))
    }
}

// Get files from a directory, considering the recursive flag
pub fn get_files_from_directory(path: &Path, flags: &Flags) -> Vec<PathBuf> {
    // Use WalkBuilder to traverse the directory
    let mut builder = WalkBuilder::new(path);
    if !flags.recursive.is_enabled() {
        builder.max_depth(Some(1)); // Limit the depth to 1 if not recursive
    }

    let files: Vec<_> = builder
        .hidden(!flags.hidden.is_enabled()) // Enable ignoring hidden files if flag is set
        .build()
        .filter_map(std::result::Result::ok) // Filter out any errors
        .filter(|entry| entry.path().is_file()) // Filter out directories
        .map(|entry| entry.path().to_path_buf()) // Convert to PathBufs
        .collect();

    files
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_files_from_path_with_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let flags = Flags::default();
        let result = get_files_from_path(file_path.to_str().unwrap(), &flags).unwrap();
        assert_eq!(result, vec![file_path]);
    }

    #[test]
    fn test_get_files_from_path_with_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let flags = Flags::default();
        let result = get_files_from_path(dir.path().to_str().unwrap(), &flags).unwrap();
        assert_eq!(result, vec![file_path]);
    }

    #[test]
    fn test_get_files_from_path_with_nonexistent_path() {
        let flags = Flags::default();
        let result = get_files_from_path("nonexistent_path", &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_files() {
        let dir = tempdir().unwrap();
        let file_path1 = dir.path().join("test_file1.txt");
        let mut file1 = File::create(&file_path1).unwrap();
        writeln!(file1, "Hello, world!").unwrap();

        let file_path2 = dir.path().join("test_file2.txt");
        let mut file2 = File::create(&file_path2).unwrap();
        writeln!(file2, "Hello, Rust!").unwrap();

        let flags = Flags::default();
        let provided = vec![dir.path().to_str().unwrap().to_string()];
        let result = get_all_files(&provided, &flags).unwrap();
        assert!(result.contains(&file_path1));
        assert!(result.contains(&file_path2));
    }

    #[test]
    fn test_get_files_from_directory_non_recursive() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file_path = subdir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut flags = Flags::default();
        flags.recursive.set_enabled(false);
        let result = get_files_from_directory(dir.path(), &flags);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_files_from_directory_recursive() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file_path = subdir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let mut flags = Flags::default();
        flags.recursive.set_enabled(true);
        let result = get_files_from_directory(dir.path(), &flags);
        assert!(result.contains(&file_path));
    }
}
