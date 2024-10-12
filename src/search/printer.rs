// src/search/printer.rs

use crate::app::flag::Flags;
use crate::search::result::SearchResult;
use colored::*;
use std::collections::HashMap;

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
pub fn print_count_results(results: &[SearchResult], flags: &Flags) {
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
pub fn print_match_results(results: &[SearchResult], flags: &Flags) {
    // Collect the results into a formatted string
    let formatted_results: Vec<String> = results
        .iter()
        .map(|res| format_match_result(res, flags))
        .collect();

    // Join the formatted results into a single string
    let output = formatted_results.join("\n");

    println!("{}", output);
}

// Format a match result for printing
pub fn format_match_result(result: &SearchResult, flags: &Flags) -> String {
    let mut output = String::new();
    use std::fmt::Write;

    // Include the file name if the flag is set
    if flags.show_names {
        write!(&mut output, "{}\n", result.file.green()).unwrap();
    }
    // Include the line number if the flag is set
    if flags.show_lines {
        write!(&mut output, "{}:", result.line_number.to_string().cyan()).unwrap();
    }

    // TODO: add separator if flag is set

    // Highlight the matches in the line content
    output.push_str(&highlight_matches(&result.line_content, &result.matches));
    output
}

// Format the count result for printing
pub fn format_count_result(file: &str, count: usize) -> String {
    format!("{}: {}", file, count)
}

// Highlight matches in a line by coloring matched text in red
pub fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
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
