// src/search/printer.rs

use crate::app::flag::Flags;
use crate::search::result::SearchMatch;
use clap::builder::Str;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Write;
use std::io::{BufWriter, Write as IoWrite};

// Print the count of matches per file
pub fn print_count_results(results: &[SearchMatch]) {
    // Create a HashMap to store counts of matches per file
    let mut file_counts: HashMap<String, usize> = HashMap::new();

    // Iterate through each match and increment the count for the file
    for result in results {
        let count = file_counts.entry(result.file.clone()).or_insert(0);
        *count += 1;
    }

    for (file, count) in file_counts.iter() {
        let output = format_count_result(file, *count);
        let stdout = std::io::stdout();
        let mut handle = BufWriter::new(stdout.lock());
        writeln!(handle, "{}", output).expect("Failed to write to stdout");
    }
}

// Print detailed match results
pub fn print_match_results(results: &[SearchMatch], flags: &Flags) {
    if results.is_empty() {
        return;
    }

    results.iter().for_each(|res| {
        let output = format_match_result(res, flags);
        let stdout = std::io::stdout();
        let mut handle = BufWriter::new(stdout.lock());
        writeln!(handle, "{}", output).expect("Failed to write to stdout");
    });
}

// Sanitize output to prevent control characters from affecting the terminal
fn sanitize_output(output: &str) -> String {
    output
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
}

// Format a match result for printing
pub fn format_match_result(result: &SearchMatch, flags: &Flags) -> String {
    let mut output = String::new();

    // Include the line number if the flag is setw
    if flags.show_lines.is_enabled() {
        write!(&mut output, "{}:", result.line_number.to_string().cyan()).unwrap();
    }

    // Sanitize the line content
    let sanitized_line_content = sanitize_output(result.line_content.as_str());
    // Highlight the matches in the line content
    output.push_str(&highlight_matches(&sanitized_line_content, &result.matches));
    output
}

// Format the count result for printing
pub fn format_count_result(file: &str, count: usize) -> String {
    format!("{}: {}", file.cyan(), count)
}

// Highlight matches in a line by coloring matched text in red
pub fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    let mut highlighted = String::with_capacity(line.len() + matches.len() * 10);
    let mut last_end = 0;

    // Iterate through each match and append highlighted text
    for (start, end) in matches {
        // Append the text before the match
        highlighted.push_str(&line[last_end..*start]);
        // Append the matched text in red
        highlighted.push_str(&line[*start..*end].red().to_string());
        last_end = *end;
    }

    highlighted.push_str(&line[last_end..]); // Append the remaining text
    highlighted
}
