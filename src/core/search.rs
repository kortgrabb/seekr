use crate::app::flag::Flags;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

const MAX_LINE_LENGTH: usize = 200;

#[derive(Debug)]
pub struct SearchResult {
    pub file: String,
    pub line_number: usize,
    pub line_content: String,
    pub matches: Vec<(usize, usize)>, // (start, end) positions of matches
}

pub fn search_files(
    needle: &str,
    files: &[String],
    flags: &Flags,
) -> Result<Vec<SearchResult>, io::Error> {
    // Get all files from the provided paths
    let files = get_all_files(files, flags)?;

    // Create a regex object, adding the ignore case flag if needed
    let regex = create_regex(needle, flags.ignore_case).expect("Invalid regex");

    if flags.parallel {
        search_files_parallel(&files, &regex)
    } else {
        search_files_sequential(&files, &regex)
    }
}

fn search_files_parallel(files: &[String], regex: &Regex) -> Result<Vec<SearchResult>, io::Error> {
    let results: Result<Vec<_>, _> = files
        .par_iter()
        .map(|file| search_file(file, regex))
        .collect();
    results.map(|vecs| vecs.into_iter().flatten().collect())
}

fn search_files_sequential(
    files: &[String],
    regex: &Regex,
) -> Result<Vec<SearchResult>, io::Error> {
    let mut results = Vec::new();
    for file in files {
        results.extend(search_file(file, regex)?);
    }
    Ok(results)
}

fn search_file(file: &str, regex: &Regex) -> Result<Vec<SearchResult>, io::Error> {
    let file_handle = File::open(file)?;
    let reader = BufReader::new(file_handle);

    let mut results: Vec<SearchResult> = Vec::new();

    // Iterate over each line in the file, searching for matches
    for (line_number, line) in reader.lines().enumerate() {
        let line = process_line(file, line_number, line, regex)?;
        if let Some(result) = line {
            results.push(result);
        }
    }

    Ok(results)
}

// Create a regex object, adding the ignore case flag if needed
fn create_regex(needle: &str, ignore_case: bool) -> Result<Regex, regex::Error> {
    let needle = if ignore_case {
        format!("(?i){}", needle)
    } else {
        needle.to_owned()
    };
    Regex::new(&needle)
}

// Process a line of text, returning a SearchResult if there are matches
fn process_line(
    file: &str,
    line_number: usize,
    line: io::Result<String>,
    regex: &Regex,
) -> Result<Option<SearchResult>, io::Error> {
    let line_content = match line {
        Ok(content) => {
            if content.len() > MAX_LINE_LENGTH {
                format!("{}...", &content[..MAX_LINE_LENGTH]) // Truncate the line
            } else {
                content
            }
        }
        Err(_) => return Ok(None), // Ignore invalid UTF-8 lines
    };

    let matches: Vec<_> = regex
        .find_iter(&line_content)
        .map(|m| (m.start(), m.end()))
        .collect();

    // If there are matches, return a SearchResult
    if !matches.is_empty() {
        Ok(Some(SearchResult {
            file: file.to_owned(),
            line_number: line_number + 1,
            line_content,
            matches,
        }))
    } else {
        Ok(None)
    }
}

// Get all files from the provided paths, handling directories and individual files
fn get_all_files(provided: &[String], flags: &Flags) -> Result<Vec<String>, io::Error> {
    let mut all_files = Vec::new();
    for path_name in provided {
        all_files.extend(get_files_from_path(path_name, flags)?);
    }

    Ok(all_files)
}

// Get files from a path, handling directories and individual files
fn get_files_from_path(path_name: &str, flags: &Flags) -> Result<Vec<String>, io::Error> {
    let path = Path::new(path_name);
    if path.is_dir() {
        get_files_from_directory(path, flags)
    } else if path.is_file() {
        Ok(vec![path_name.to_owned()])
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{}' is neither a file nor a directory.", path_name),
        ))
    }
}

// Get files from a directory, handling recursion if needed
fn get_files_from_directory(path: &Path, flags: &Flags) -> Result<Vec<String>, io::Error> {
    let files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok) // Filter out any errors from walking the directory
        .filter(|entry| entry.file_type().is_file() && (flags.recursive || entry.depth() == 1)) // Only include files, and handle recursion
        .filter_map(|entry| entry.path().to_str().map(String::from)) // Convert the path to a String
        .collect();
    Ok(files)
}

// Print the search results based on the provided flags
pub fn print_results(results: &[SearchResult], flags: &Flags) {
    if flags.count {
        print_count_results(results, flags);
    } else {
        print_match_results(results, flags);
    }
}

// Print the count of matches per file
fn print_count_results(results: &[SearchResult], flags: &Flags) {
    let file_counts = results.iter().fold(HashMap::new(), |mut acc, res| {
        *acc.entry(&res.file).or_insert(0) += 1;
        acc
    });

    if flags.show_names {
        for (file, count) in file_counts {
            println!("{}", format_count_result(file, count));
        }
    } else {
        let total_count: usize = file_counts.values().sum();
        println!("{}", total_count);
    }
}

// Print the match results for each file
fn print_match_results(results: &[SearchResult], flags: &Flags) {
    for result in results {
        println!("{}", format_match_result(result, flags));
    }
}

// Format the match result for printing
fn format_match_result(result: &SearchResult, flags: &Flags) -> String {
    let mut output = String::new();
    use std::fmt::Write; // Import the write trait

    // write directly into the String
    if flags.show_names {
        write!(&mut output, "{}:", result.file).unwrap(); // Write directly into the String
    }
    if flags.show_lines {
        write!(&mut output, "{}:", result.line_number).unwrap();
    }

    output.push_str(&highlight_matches(&result.line_content, &result.matches));
    output
}

// Format the count result for printing
fn format_count_result(file: &str, count: usize) -> String {
    format!("{}: {}", file, count)
}

// Highlight matches in a line of text
fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    let mut highlighted = String::with_capacity(line.len() + matches.len() * 10); // Rough estimate of the final length
    let mut last_end = 0;

    for &(start, end) in matches {
        // Append the text before the match
        highlighted.push_str(&line[last_end..start]);
        // Append the match itself, highlighted in red
        highlighted.push_str(&line[start..end].red().to_string());
        // Update the last end position
        last_end = end;
    }
    // Append the text after the last match
    highlighted.push_str(&line[last_end..]);
    highlighted
}
