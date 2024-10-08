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
    let files = match get_all_files(files, flags) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error getting files: {}", e);
            return Vec::new();
        }
    };
    let regex = match create_regex(needle, flags.ignore_case) {
        Ok(regex) => regex,
        Err(e) => {
            eprintln!("Invalid regex pattern: {}", e);
            return Vec::new();
        }
    };

    if flags.parallel {
        files
            .par_iter()
            .flat_map(|file| match search_file(file, &regex) {
                Ok(results) => results,
                Err(e) => {
                    eprintln!("Error searching file {}: {}", file, e);
                    Vec::new()
                }
            })
            .collect()
    } else {
        files
            .iter()
            .flat_map(|file| match search_file(file, &regex) {
                Ok(results) => results,
                Err(e) => {
                    eprintln!("Error searching file {}: {}", file, e);
                    Vec::new()
                }
            })
            .collect()
    }
}

fn search_file(file: &str, regex: &Regex) -> Result<Vec<SearchResult>, io::Error> {
    let file_handle = File::open(file)?;
    let reader = io::BufReader::new(file_handle);

    let results = reader
        .lines()
        .enumerate()
        .filter_map(
            |(line_number, line)| match process_line(file, line_number, line, regex) {
                Ok(Some(result)) => Some(result),
                Ok(None) => None,
                Err(e) => {
                    eprintln!(
                        "Error processing line {} in file {}: {}",
                        line_number + 1,
                        file,
                        e
                    );
                    None
                }
            },
        )
        .collect();
    Ok(results)
}

fn create_regex(needle: &str, ignore_case: bool) -> Result<Regex, regex::Error> {
    let needle = if ignore_case {
        format!("(?i){}", needle)
    } else {
        needle.to_string()
    };
    Regex::new(&needle)
}

fn process_line(
    file: &str,
    line_number: usize,
    line: io::Result<String>,
    regex: &Regex,
) -> Result<Option<SearchResult>, io::Error> {
    let line_content = line?;
    let matches: Vec<_> = regex
        .find_iter(&line_content)
        .map(|m| (m.start(), m.end()))
        .collect();

    if !matches.is_empty() {
        Ok(Some(SearchResult {
            file: file.to_string(),
            line_number: line_number + 1,
            line_content,
            matches,
        }))
    } else {
        Ok(None)
    }
}

fn get_all_files(provided: &[String], flags: &Flags) -> Result<Vec<String>, io::Error> {
    let mut all_files = Vec::new();
    for path_name in provided {
        all_files.extend(get_files_from_path(path_name, flags)?);
    }
    Ok(all_files)
}

fn get_files_from_path(path_name: &str, flags: &Flags) -> Result<Vec<String>, io::Error> {
    let path = Path::new(path_name);
    if path.is_dir() {
        let files = WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file() && (flags.recursive || entry.depth() == 1))
            .filter_map(|entry| entry.path().to_str().map(String::from))
            .collect();
        Ok(files)
    } else if path.is_file() {
        Ok(vec![path_name.to_string()])
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{}' is neither a file nor a directory.", path_name),
        ))
    }
}

pub fn print_results(results: &[SearchResult], flags: &Flags) {
    if flags.count {
        print_count_results(results, flags);
    } else {
        print_match_results(results, flags);
    }
}

fn print_count_results(results: &[SearchResult], flags: &Flags) {
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
        println!("{}", total_count);
    }
}

fn print_match_results(results: &[SearchResult], flags: &Flags) {
    for result in results {
        println!("{}", format_match_result(result, flags));
    }
}

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

fn format_count_result(file: &str, count: usize, _flags: &Flags) -> String {
    format!("{}: {}", file, count)
}

fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    let mut highlighted = String::new();
    let mut last_end = 0;
    for &(start, end) in matches {
        highlighted.push_str(&line[last_end..start]);
        highlighted.push_str(&line[start..end].red().to_string());
        last_end = end;
    }
    highlighted.push_str(&line[last_end..]);
    highlighted
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_files_single_file() {
        let flags = Flags {
            ignore_case: false,
            parallel: false,
            recursive: false,
            count: false,
            show_names: false,
            show_lines: false,
            invert_match: false,
        };
        let files = vec!["test_data/test_file.txt".to_string()];
        let results = search_files("test", &files, &flags);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].file, "test_data/test_file.txt");
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[0].line_content, "This is a test file.");
        assert_eq!(results[0].matches.len(), 1);
    }

    #[test]
    fn test_search_files_multiple_files() {
        let flags = Flags {
            ignore_case: false,
            parallel: false,
            recursive: false,
            count: false,
            show_names: false,
            show_lines: false,
            invert_match: false,
        };
        let files = vec![
            "test_data/test_file.txt".to_string(),
            "test_data/another_test_file.txt".to_string(),
        ];
        let results = search_files("test", &files, &flags);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_files_ignore_case() {
        let flags = Flags {
            ignore_case: true,
            parallel: false,
            recursive: false,
            count: false,
            show_names: false,
            show_lines: false,
            invert_match: false,
        };
        let files = vec!["test_data/test_file.txt".to_string()];
        let results = search_files("TEST", &files, &flags);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_files_case_sensitive() {
        let flags = Flags {
            ignore_case: false,
            parallel: false,
            recursive: false,
            count: false,
            show_names: false,
            show_lines: false,
            invert_match: false,
        };
        let files = vec!["test_data/test_file.txt".to_string()];
        let results = search_files("TEST", &files, &flags);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_files_parallel() {
        let flags = Flags {
            ignore_case: false,
            parallel: true,
            recursive: false,
            count: false,
            show_names: false,
            show_lines: false,
            invert_match: false,
        };
        let files = vec![
            "test_data/test_file.txt".to_string(),
            "test_data/another_test_file.txt".to_string(),
        ];
        let results = search_files("test", &files, &flags);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_print_results_count() {
        let flags = Flags {
            ignore_case: false,
            parallel: false,
            recursive: false,
            count: true,
            show_names: true,
            show_lines: false,
            invert_match: false,
        };
        let results = vec![
            SearchResult {
                file: "test_data/test_file.txt".to_string(),
                line_number: 1,
                line_content: "This is a test file.".to_string(),
                matches: vec![(10, 14)],
            },
            SearchResult {
                file: "test_data/test_file.txt".to_string(),
                line_number: 2,
                line_content: "Another test line.".to_string(),
                matches: vec![(8, 12)],
            },
        ];
        print_results(&results, &flags);
    }

    #[test]
    fn test_print_results_match() {
        let flags = Flags {
            ignore_case: false,
            parallel: false,
            recursive: false,
            count: false,
            show_names: true,
            show_lines: true,
            invert_match: false,
        };
        let results = vec![SearchResult {
            file: "test_data/test_file.txt".to_string(),
            line_number: 1,
            line_content: "This is a test file.".to_string(),
            matches: vec![(10, 14)],
        }];
        print_results(&results, &flags);
    }
}
