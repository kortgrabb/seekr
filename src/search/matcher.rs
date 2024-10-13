use crate::app::flag::Flags;
use crate::search::file_io::get_all_files;
use crate::search::result::SearchMatch;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use super::printer::print_match_results;

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

// // Main function to search for a pattern in a list of files
// pub fn search_multiple_files(
//     needle: &str,
//     files: &[PathBuf],
//     flags: &Flags,
// ) -> Result<Vec<SearchMatch>, io::Error> {
//     let regex = create_regex(needle, flags.ignore_case.is_enabled())
//         .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

//     let file_count_threshold = 10;
//     let total_size_threshold = 10_000_000; // ~10 MB

//     let total_size: u64 = files
//         .iter()
//         .filter_map(|f| fs::metadata(f).ok()) // Filter out files with invalid metadata
//         .map(|metadata| metadata.len())
//         .sum();

//     // Decide whether to search files in parallel or sequentially
//     // Based on the number of files and size
//     let use_parallel = files.len() > file_count_threshold || total_size > total_size_threshold;

//     if use_parallel {
//         search_files_parallel(files, &regex, flags)
//     } else {
//         search_files_sequential(files, &regex, flags)
//     }
// }

pub fn search_single_file(
    needle: &str,
    file: &str,
    flags: &Flags,
) -> Result<Vec<SearchMatch>, io::Error> {
    let regex = create_regex(needle, flags.ignore_case.is_enabled())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let file = PathBuf::from(file);
    search_file(&file, &regex, flags)
}

// // Search files in parallel using Rayon
// pub fn search_files_parallel(
//     files: &[PathBuf],
//     regex: &Regex,
//     flags: &Flags,
// ) -> Result<Vec<SearchMatch>, io::Error> {
//     // Use Rayon to search files in parallel, which can speed up the search for large file sets
//     let results: Result<Vec<_>, _> = files
//         .par_iter()
//         .map(|file| {
//             let matches = search_file(file, regex, flags)?;
//             if !matches.is_empty() {
//                 print_match_results(&matches, flags);
//             }
//             Ok(matches)
//         })
//         .collect();
//     results.map(|vecs| vecs.into_iter().flatten().collect())
// }

// // Search files sequentially
// pub fn search_files_sequential(
//     files: &[PathBuf],
//     regex: &Regex,
//     flags: &Flags,
// ) -> Result<Vec<SearchMatch>, io::Error> {
//     let mut results = Vec::new();
//     // Iterate through each file and search for matches
//     for file in files {
//         let matches = search_file(file, regex, flags)?;
//         if !matches.is_empty() {
//             print_match_results(&matches, flags);
//         }
//         results.extend(matches);
//     }
//     Ok(results)
// }

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
            print_match_results(&results, flags);
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
