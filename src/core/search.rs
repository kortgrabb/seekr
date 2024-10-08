use crate::app::flag::Flags;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct SearchResult {
    pub file: String,
    pub line_number: usize,
    pub line_content: String,
    pub matches: Vec<(usize, usize)>, // (start, end) positions of matches
}

pub fn search_files(needle: &str, files: &[String], flags: &Flags) -> Vec<SearchResult> {
    let files = get_all_files(files, flags);
    let regex = create_regex(needle, flags.ignore_case);

    // Iterate over files in either parallel or sequential mode. Then, flatten the results.
    if flags.parallel {
        files
            .par_iter()
            .flat_map(|file| search_file(file, &regex))
            .collect()
    } else {
        files
            .iter()
            .flat_map(|file| search_file(file, &regex))
            .collect()
    }
}

fn search_file(file: &str, regex: &Regex) -> Vec<SearchResult> {
    let file_handle = File::open(file).expect("Failed to open file");
    let reader = io::BufReader::new(file_handle);

    // Enumerate lines to get line numbers, then filter and process each line.
    reader
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| process_line(file, line_number, line, regex))
        .collect()
}

// Create a regex needle with the given flags.
fn create_regex(needle: &str, ignore_case: bool) -> Regex {
    let needle = if ignore_case {
        format!("(?i){}", needle)
    } else {
        needle.to_string()
    };
    Regex::new(&needle).expect("Invalid regex pattern")
}

// Process a line of text, returning a SearchResult if the line contains a match.
fn process_line(
    file: &str,
    line_number: usize,
    line: io::Result<String>,
    regex: &Regex,
) -> Option<SearchResult> {
    line.ok().and_then(|line_content| {
        let matches: Vec<_> = regex
            .find_iter(&line_content)
            .map(|m| (m.start(), m.end()))
            .collect();

        // Return a SearchResult if there are matches, otherwise None.
        if !matches.is_empty() {
            Some(SearchResult {
                file: file.to_string(),
                line_number: line_number + 1,
                line_content,
                matches,
            })
        } else {
            None
        }
    })
}

// Get all files from a list of paths, filtering out directories and non-existent files.
fn get_all_files(provided: &[String], flags: &Flags) -> Vec<String> {
    provided
        .iter()
        .flat_map(|path_name| get_files_from_path(path_name, flags))
        .collect()
}

// Get files from a path, filtering out directories and non-existent files.
fn get_files_from_path(path_name: &str, flags: &Flags) -> Vec<String> {
    let path = Path::new(path_name);
    if path.is_dir() {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file() && (flags.recursive || entry.depth() == 1))
            .filter_map(|entry| entry.path().to_str().map(String::from))
            .collect()
    } else if path.is_file() {
        vec![path_name.to_string()]
    } else {
        eprintln!(
            "Warning: '{}' is neither a file nor a directory.",
            path_name
        );
        Vec::new()
    }
}

// Print search results based on the flags provided.
pub fn print_results(results: &[SearchResult], flags: &Flags) {
    if flags.count {
        print_count_results(results, flags);
    } else {
        print_match_results(results, flags);
    }
}

// Print the count of matches per file.
fn print_count_results(results: &[SearchResult], flags: &Flags) {
    // Use fold to accumulate counts into a HashMap
    let file_counts = results.iter().fold(HashMap::new(), |mut acc, res| {
        *acc.entry(&res.file).or_insert(0) += 1;
        acc
    });

    if flags.show_names {
        for (file, count) in file_counts {
            println!("{}", format_count_result(file, count, flags));
        }
    } else {
        let total_count: usize = file_counts.values().sum();
        println!("total matches: {}", total_count);
    }
}

// Print the match results based on the flags provided.
fn print_match_results(results: &[SearchResult], flags: &Flags) {
    for result in results {
        // Print the formatted result based on the flags.
        println!("{}", format_match_result(result, flags));
    }
}

// Format a SearchResult based on the flags provided.
fn format_match_result(result: &SearchResult, flags: &Flags) -> String {
    let mut output = String::new();

    if flags.show_names {
        output.push_str(&format!("{}:", result.file));
    }
    if flags.show_lines {
        output.push_str(&format!("{}:", result.line_number));
    }

    output.push_str(&highlight_matches(&result.line_content, &result.matches));
    output
}

// Format the count result for a file.
fn format_count_result(file: &str, count: usize, flags: &Flags) -> String {
    let mut output = String::new();

    if flags.show_names {
        output.push_str(&format!("{}:", file));
    }

    output.push_str(&format!(" {}", count));
    output
}

// Highlight matches in a line of text.
fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    let mut highlighted = String::new();
    let mut last_end = 0;
    for &(start, end) in matches {
        // Append the text that is before the match.
        highlighted.push_str(&line[last_end..start]);
        // Append the match itself, highlighted in red.
        highlighted.push_str(&line[start..end].red().to_string());
        // Update the last end position.
        last_end = end;
    }
    // Append the text that is after the last match.
    highlighted.push_str(&line[last_end..]);
    highlighted
}
