use crate::app::flag::Flags;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Limit the maximum length of a line to avoid processing overly large lines
const MAX_LINE_LENGTH: usize = 200;

#[derive(Debug)]
pub struct SearchResult {
    pub file: String,                 // Name of the file containing the match
    pub line_number: usize,           // Line number where the match was found
    pub line_content: String,         // Content of the matched line
    pub matches: Vec<(usize, usize)>, // (start, end) positions of matches
}

impl SearchResult {
    // Creates a new SearchResult instance with file name, line number, content, and match positions
    pub fn new(
        file: &str,
        line_number: usize,
        line_content: String,
        matches: Vec<(usize, usize)>,
    ) -> Self {
        Self {
            file: file.to_owned(),
            line_number,
            line_content,
            matches,
        }
    }
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
        search_files_parallel(&files, &regex)
    } else {
        search_files_sequential(&files, &regex)
    }
}

// Search files in parallel using Rayon
fn search_files_parallel(files: &[PathBuf], regex: &Regex) -> Result<Vec<SearchResult>, io::Error> {
    // Use Rayon to search files in parallel, which can speed up the search for large file sets
    let results: Result<Vec<_>, _> = files
        .par_iter()
        .map(|file| search_file(file, regex))
        .collect();
    results.map(|vecs| vecs.into_iter().flatten().collect())
}

// Search files sequentially
fn search_files_sequential(
    files: &[PathBuf],
    regex: &Regex,
) -> Result<Vec<SearchResult>, io::Error> {
    let mut results = Vec::new();
    // Iterate through each file and search for matches
    for file in files {
        results.extend(search_file(file, regex)?);
    }
    Ok(results)
}

// Search for matches in a specific file
fn search_file(file: &Path, regex: &Regex) -> Result<Vec<SearchResult>, io::Error> {
    // Open the file for reading
    let file_handle = File::open(file)?;
    let reader = BufReader::new(file_handle);

    let mut results: Vec<SearchResult> = Vec::new();
    // Iterate through each line in the file
    for (line_number, line) in reader.lines().enumerate() {
        // Process each line to find matches
        let line = process_line(file, line_number, line, regex)?;
        if let Some(result) = line {
            results.push(result);
        }
    }

    Ok(results)
}

// Create a regex pattern from the search string, handling case sensitivity
fn create_regex(needle: &str, ignore_case: bool) -> Result<Regex, regex::Error> {
    // If case-insensitive search is requested, prepend the regex with "(?i)"
    let needle = if ignore_case {
        format!("(?i){}", needle)
    } else {
        needle.to_owned()
    };
    Regex::new(&needle)
}

// Process a line to find matches and truncate if necessary
fn process_line(
    file: &Path,
    line_number: usize,
    line: io::Result<String>,
    regex: &Regex,
) -> Result<Option<SearchResult>, io::Error> {
    let line_content = match line {
        // Truncate line if it's too long to avoid processing overly large lines
        Ok(content) => {
            if content.len() > MAX_LINE_LENGTH {
                let valid_truncation_index = content
                    .char_indices()
                    .take(MAX_LINE_LENGTH)
                    .last()
                    .map(|(idx, _)| idx)
                    .unwrap_or(content.len());
                format!("{}...", &content[..valid_truncation_index])
            } else {
                content
            }
        }
        Err(_) => return Ok(None),
    };

    // Find matches in the line content using the regex
    let matches: Vec<_> = regex
        .find_iter(&line_content)
        .map(|m| (m.start(), m.end()))
        .collect();

    // Return a SearchResult if matches are found
    if !matches.is_empty() {
        Ok(Some(SearchResult::new(
            file.to_string_lossy().as_ref(),
            line_number + 1, // Line numbers are 1-based
            line_content,
            matches,
        )))
    } else {
        Ok(None)
    }
}

// Get all files from a list of provided paths
fn get_all_files(provided: &[String], flags: &Flags) -> Result<Vec<PathBuf>, io::Error> {
    let mut all_files = Vec::new();
    // Iterate through each provided path and collect all files
    for path_name in provided {
        all_files.extend(get_files_from_path(path_name, flags)?);
    }

    Ok(all_files)
}

// Get files from a specific path, handling directories and files
fn get_files_from_path(path_name: &str, flags: &Flags) -> Result<Vec<PathBuf>, io::Error> {
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
fn get_files_from_directory(path: &Path, flags: &Flags) -> Result<Vec<PathBuf>, io::Error> {
    // Use WalkDir to traverse the directory
    let files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok) // Filter out any errors
        .filter(|entry| entry.file_type().is_file() && (flags.recursive || entry.depth() == 1))
        .map(|entry| entry.into_path())
        .collect();
    Ok(files)
}

// Print search results, either count or detailed matches
pub fn print_results(results: &[SearchResult], flags: &Flags) {
    if flags.count {
        // Print the count of matches per file
        print_count_results(results, flags);
    } else {
        // Print detailed match results
        print_match_results(results, flags);
    }
}

// Print the count of matches per file
fn print_count_results(results: &[SearchResult], flags: &Flags) {
    // Create a HashMap to store counts of matches per file
    let file_counts = results.iter().fold(HashMap::new(), |mut acc, res| {
        *acc.entry(&res.file).or_insert(0) += 1;
        acc
    });

    if flags.show_names {
        // Print the count for each file if file names should be shown
        for (file, count) in file_counts {
            println!("{}", format_count_result(file, count));
        }
    } else {
        // Print the total count of matches
        let total_count: usize = file_counts.values().sum();
        println!("{}", total_count);
    }
}

// Print detailed match results
fn print_match_results(results: &[SearchResult], flags: &Flags) {
    // Iterate through each SearchResult and print the formatted match result
    for result in results {
        println!("{}", format_match_result(result, flags));
    }
}

// Format a match result for printing
fn format_match_result(result: &SearchResult, flags: &Flags) -> String {
    let mut output = String::new();
    use std::fmt::Write;

    // Include the file name if the flag is set
    if flags.show_names {
        write!(&mut output, "{}:", result.file).unwrap();
    }
    // Include the line number if the flag is set
    if flags.show_lines {
        write!(&mut output, "{}:", result.line_number).unwrap();
    }

    // Highlight the matches in the line content
    output.push_str(&highlight_matches(&result.line_content, &result.matches));
    output
}

// Format the count result for printing
fn format_count_result(file: &str, count: usize) -> String {
    format!("{}: {}", file, count)
}

// Highlight matches in a line by coloring matched text in red
fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    let mut highlighted = String::with_capacity(line.len() + matches.len() * 10);
    let mut last_end = 0;

    // Iterate through each match and append highlighted text
    for &(start, end) in matches {
        highlighted.push_str(&line[last_end..start]); // Append text before the match
        highlighted.push_str(&line[start..end].red().to_string()); // Append the matched text in red
        last_end = end;
    }
    highlighted.push_str(&line[last_end..]); // Append the remaining text
    highlighted
}
