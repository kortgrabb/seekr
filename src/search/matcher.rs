use crate::app::flag::Flags;
use crate::search::result::SearchMatch;
use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use super::printer::{print_count_results, print_match_results};

// TODO: add non-regex search if not needed

// Create a regex pattern from the search string, handling case sensitivity
lazy_static! {
    static ref REGEX_CACHE: Mutex<HashMap<(String, bool), Regex>> = Mutex::new(HashMap::new());
}

pub fn create_regex(needle: &str, ignore_case: bool) -> Result<Regex, regex::Error> {
    let key = (needle.to_string(), ignore_case);
    let mut cache = REGEX_CACHE.lock().unwrap();

    if let Some(regex) = cache.get(&key) {
        return Ok(regex.clone());
    }

    let needle = if ignore_case {
        format!("(?i){needle}")
    } else {
        needle.to_owned()
    };

    let regex = Regex::new(&needle)?;
    cache.insert(key, regex.clone());
    Ok(regex)
}

pub fn search_single_file(needle: &str, file: &str, flags: &Flags) -> Result<usize, io::Error> {
    let regex = create_regex(needle, flags.ignore_case.is_enabled())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let file = PathBuf::from(file);
    search_file(&file, &regex, flags)
}

// Search for matches in a specific file
pub fn search_file(file: &Path, regex: &Regex, flags: &Flags) -> Result<usize, io::Error> {
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

        // Add the match to the results if it exists
        if let Some(result) = line {
            results.push(result);
        }
    }

    if !results.is_empty() && !flags.count.is_enabled() {
        println!("{}", file.to_string_lossy().blue());
        print_match_results(&results, flags);
    } else if flags.count.is_enabled() {
        print_count_results(&results);
    }
    let total_count = results.len();
    Ok(total_count)
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
