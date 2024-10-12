use crate::app::flag::Flags;
use crate::search::file_io::get_all_files;
use crate::search::result::SearchResult;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

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
) -> Result<Vec<SearchResult>, io::Error> {
    // Get all files to be searched
    let files = get_all_files(files, flags)?;
    // Create a regex pattern, considering case sensitivity
    let regex = create_regex(needle, flags.ignore_case)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Choose between parallel or sequential file searching based on the flag
    if flags.parallel {
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
) -> Result<Vec<SearchResult>, io::Error> {
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
) -> Result<Vec<SearchResult>, io::Error> {
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
) -> Result<Vec<SearchResult>, io::Error> {
    // Open the file for reading
    let file_handle = File::open(file)?;
    let reader = BufReader::new(file_handle);

    let mut results: Vec<SearchResult> = Vec::new();
    // Iterate through each line in the file
    for (line_number, line) in reader.lines().enumerate() {
        // Process each line to find matches
        let line = process_line(file, line_number, line, regex, flags.invert_match)?;
        if let Some(result) = line {
            results.push(result);
        }
    }

    Ok(results)
}

/// Processes a single line from a file to determine if it matches (or does not match) the given regex.
///
/// # Arguments
///
/// * `file` - The path to the file being processed.
/// * `line_number` - The current line number in the file.
/// * `line` - The content of the line, wrapped in an `io::Result`.
/// * `regex` - The compiled regular expression to match against.
/// * `invert_match` - If `true`, lines that do NOT match the regex are considered matches.
///
/// # Returns
///
/// * `Ok(Some(SearchResult))` if the line is a match based on `invert_match`.
/// * `Ok(None)` if the line is not a match.
/// * `Err(io::Error)` if an I/O error occurs while reading the line.
pub fn process_line(
    file: &Path,
    line_number: usize,
    line: io::Result<String>,
    regex: &Regex,
    invert_match: bool,
) -> Result<Option<SearchResult>, io::Error> {
    let line_content = match line {
        Ok(content) => content,
        Err(_) => {
            return Ok(None); // Skip the line if there's an error reading it
        }
    };

    // Find matches in the line content using the regex
    let matches: Vec<_> = regex.find_iter(&line_content).collect();

    if invert_match {
        if matches.is_empty() {
            // Line does NOT match the regex; it's a match for inverted search
            Ok(Some(SearchResult::new(
                file.to_string_lossy().as_ref(),
                line_number + 1, // Line numbers are 1-based
                line_content,
                Vec::new(), // No matches since we're inverting      // Indicate this is an inverted match
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
        Ok(Some(SearchResult::new(
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
