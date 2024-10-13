use crate::app::flag::Flags;
use crate::search::file_io::get_all_files;
use crate::search::result::SearchMatch;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

// TODO: add non-regex search if not needed

// Create a regex pattern from the search string, handling case sensitivity
pub fn create_regex(needle: &str, ignore_case: bool) -> Result<Regex, regex::Error> {
    // If case-insensitive search is requested, prepend the regex with "(?i)"
    let needle = if ignore_case {
        format!("(?i){needle}")
    } else {
        needle.to_owned()
    };
    Regex::new(&needle)
}

// Main function to search for a pattern in a list of files
pub fn search_files(
    needle: &str,
    files: &[String],
    flags: &Flags,
) -> Result<Vec<SearchMatch>, io::Error> {
    // Get all files to be searched
    let files = get_all_files(files, flags)?;
    // Create a regex pattern, considering case sensitivity
    let regex = create_regex(needle, flags.ignore_case.is_enabled())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Choose between parallel or sequential file searching based on the flag
    if flags.parallel.is_enabled() {
        search_files_parallel(&files, &regex, flags)
    } else {
        search_files_sequential(&files, &regex, flags)
    }
}

// Search files in parallel using Rayon
pub fn search_files_parallel(
    files: &[PathBuf],
    regex: &Regex,
    flags: &Flags,
) -> Result<Vec<SearchMatch>, io::Error> {
    // Use Rayon to search files in parallel, which can speed up the search for large file sets
    let results: Result<Vec<_>, _> = files
        .par_iter()
        .map(|file| search_file(file, regex, flags))
        .collect();
    results.map(|vecs| vecs.into_iter().flatten().collect())
}

// Search files sequentially
pub fn search_files_sequential(
    files: &[PathBuf],
    regex: &Regex,
    flags: &Flags,
) -> Result<Vec<SearchMatch>, io::Error> {
    let mut results = Vec::new();
    // Iterate through each file and search for matches
    for file in files {
        results.extend(search_file(file, regex, flags)?);
    }
    Ok(results)
}

// Search for matches in a specific file
pub fn search_file(
    file: &Path,
    regex: &Regex,
    flags: &Flags,
) -> Result<Vec<SearchMatch>, io::Error> {
    // Open the file for reading
    let file_handle = File::open(file)?;
    let reader = BufReader::new(file_handle);

    let mut results: Vec<SearchMatch> = Vec::new();
    // Iterate through each line in the file
    for (line_number, line) in reader.lines().enumerate() {
        // Process each line to find matches
        let line = process_line(
            file,
            line_number,
            line,
            regex,
            flags.invert_match.is_enabled(),
        )?;
        if let Some(result) = line {
            results.push(result);
        }
    }

    Ok(results)
}
pub fn process_line(
    file: &Path,
    line_number: usize,
    line: io::Result<String>,
    regex: &Regex,
    invert_match: bool,
) -> Result<Option<SearchMatch>, io::Error> {
    // Handle potential invalid UTF-8 sequences gracefully
    let line_content = match line {
        Ok(content) => content,
        Err(_) => return Ok(None), // Skip lines with invalid UTF-8
    };

    // Find matches in the line content using the regex
    let matches: Vec<_> = regex.find_iter(&line_content).collect();

    if invert_match {
        if matches.is_empty() {
            // Line does NOT match the regex; it's a match for inverted search
            Ok(Some(SearchMatch::new(
                file.to_string_lossy().as_ref(),
                line_number + 1, // Line numbers are 1-based
                line_content,
                Vec::new(), // No matches since we're inverting
            )))
        } else {
            // Line matches the regex; skip it in inverted search
            Ok(None)
        }
    } else if !matches.is_empty() {
        // Line matches the regex; include it in the results
        let match_positions = matches
            .iter()
            .map(|m| (m.start(), m.end()))
            .collect::<Vec<(usize, usize)>>();
        Ok(Some(SearchMatch::new(
            file.to_string_lossy().as_ref(),
            line_number + 1, // Line numbers are 1-based
            line_content,
            match_positions,
        )))
    } else {
        // Line does not match the regex; skip it
        Ok(None)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;

    // test_file.txt: "This is a test line.\nAnother test line."

    fn create_test_file(content: &str) -> PathBuf {
        let file_path = PathBuf::from("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{content}").unwrap();
        file_path
    }

    #[test]
    fn test_create_regex_case_sensitive() {
        let regex = create_regex("test", false).unwrap();
        assert!(regex.is_match("test"));
        assert!(!regex.is_match("TEST"));
    }

    #[test]
    fn test_create_regex_case_insensitive() {
        let regex = create_regex("test", true).unwrap();
        assert!(regex.is_match("test"));
        assert!(regex.is_match("TEST"));
    }

    #[test]
    fn test_search_file() {
        let file_path = create_test_file("This is a test line.\nAnother test line.");
        let regex = create_regex("test", false).unwrap();
        let flags = Flags::default();
        let matches = search_file(&file_path, &regex, &flags).unwrap();
        assert_eq!(matches.len(), 2);
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_search_file_invert_match() {
        let file_path = create_test_file("This is a test line.\nAnother test line.");
        let regex = create_regex("test", false).unwrap();
        let mut flags = Flags::default();
        flags.invert_match.set_enabled(true);
        let matches = search_file(&file_path, &regex, &flags).unwrap();
        assert_eq!(matches.len(), 0);
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_process_line_match() {
        let file_path = PathBuf::from("test_file.txt");
        let regex = create_regex("test", false).unwrap();
        let line = Ok("This is a test line.".to_string());
        let result = process_line(&file_path, 0, line, &regex, false).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_process_line_no_match() {
        let file_path = PathBuf::from("test_file.txt");
        let regex = create_regex("test", false).unwrap();
        let line = Ok("This is a line.".to_string());
        let result = process_line(&file_path, 0, line, &regex, false).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_process_line_invert_match() {
        let file_path = PathBuf::from("test_file.txt");
        let regex = create_regex("test", false).unwrap();
        let line = Ok("This is a line.".to_string());
        let result = process_line(&file_path, 0, line, &regex, true).unwrap();
        assert!(result.is_some());
    }
}
