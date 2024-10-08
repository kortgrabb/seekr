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

pub fn search_files(pattern: &str, files: &[String], flags: &Flags) -> Vec<SearchResult> {
    let files = get_all_files(files, flags);
    let regex = create_regex(pattern, flags.ignore_case);

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

    reader
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| process_line(file, line_number, line, regex))
        .collect()
}

fn create_regex(pattern: &str, ignore_case: bool) -> Regex {
    let pattern = if ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    Regex::new(&pattern).expect("Invalid regex pattern")
}

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

fn get_all_files(provided: &[String], flags: &Flags) -> Vec<String> {
    provided
        .iter()
        .flat_map(|path_name| get_files_from_path(path_name, flags))
        .collect()
}

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

pub fn print_results(results: &[SearchResult], flags: &Flags) {
    if flags.count {
        print_count_results(results);
    } else {
        print_match_results(results, flags);
    }
}

fn print_count_results(results: &[SearchResult]) {
    let file_counts: HashMap<_, _> =
        results
            .iter()
            .map(|r| &r.file)
            .fold(HashMap::new(), |mut acc, file| {
                *acc.entry(file).or_insert(0) += 1;
                acc
            });

    for (file, count) in file_counts {
        println!("{}: {}", file, count);
    }
}

fn print_match_results(results: &[SearchResult], flags: &Flags) {
    for result in results {
        println!("{}", format_result(result, flags));
    }
}

fn format_result(result: &SearchResult, flags: &Flags) -> String {
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

fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    let mut highlighted = String::new();
    let mut last_end = 0;
    for &(start, end) in matches {
        highlighted.push_str(&line[last_end..start]);
        highlighted.push_str(&line[start..end].cyan().to_string());
        last_end = end;
    }
    highlighted.push_str(&line[last_end..]);
    highlighted
}
