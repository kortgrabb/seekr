// src/search/printer.rs

use crate::app::flag::Flags;
use crate::plugin_integration::lua_plugin::LuaPlugin;
use crate::search::result::SearchMatch;
use colored::Colorize;
use rlua::{Lua, Result as LuaResult, RluaCompat};
use std::collections::HashMap;
use std::fmt::Write;
use std::io::{BufWriter, Write as IoWrite};

// Print search results, either count or detailed matches
pub fn print_results(results: &[SearchMatch], flags: &Flags) {
    if flags.count.is_enabled() {
        // Print the count of matches per file
        print_count_results(results, flags);
    } else {
        // Print detailed match results
        print_match_results(results, flags);
    }
}

// Print the count of matches per file
pub fn print_count_results(results: &[SearchMatch], flags: &Flags) {
    // Create a HashMap to store counts of matches per file
    let file_counts = results.iter().fold(HashMap::new(), |mut acc, res| {
        *acc.entry(&res.file).or_insert(0) += 1;
        acc
    });

    if flags.show_names.is_enabled() {
        // Print the count for each file if file names should be shown
        for (file, count) in file_counts {
            println!("{}", format_count_result(file, count));
        }
    } else {
        // Print the total count of matches
        let total_count: usize = file_counts.values().sum();
        println!("{total_count}");
    }
}

// Print detailed match results
pub fn print_match_results(results: &[SearchMatch], flags: &Flags) {
    if results.is_empty() {
        return;
    }

    results
        .iter()
        .map(|res| format_match_result(res, flags))
        .for_each(|res| {
            let stdout = std::io::stdout();
            let mut handle = BufWriter::new(stdout.lock());
            writeln!(handle, "{}", res).expect("Failed to write to stdout");
        });
}

// Format a match result for printing
pub fn format_match_result(result: &SearchMatch, flags: &Flags) -> String {
    let mut output = String::new();

    // Include the file name if the flag is set
    if flags.show_names.is_enabled() {
        writeln!(&mut output, "{}", result.file.green()).unwrap();
    }
    // Include the line number if the flag is set
    if flags.show_lines.is_enabled() {
        write!(&mut output, "{}:", result.line_number.to_string().cyan()).unwrap();
    }

    // TODO: add separator if flag is set

    // Highlight the matches in the line content
    output.push_str(&highlight_matches(&result.line_content, &result.matches));
    output
}

// Format the count result for printing
pub fn format_count_result(file: &str, count: usize) -> String {
    format!("{file}: {count}")
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

#[cfg(test)]
mod tests {
    use crate::app::flag::OptionState;

    use super::*;

    #[test]
    fn test_print_count_results_with_file_names() {
        let results = vec![
            SearchMatch {
                file: "file1.txt".to_string(),
                line_number: 1,
                line_content: "This is a test".to_string(),
                matches: vec![(0, 4)],
            },
            SearchMatch {
                file: "file2.txt".to_string(),
                line_number: 2,
                line_content: "Another test".to_string(),
                matches: vec![(8, 12)],
            },
            SearchMatch {
                file: "file1.txt".to_string(),
                line_number: 3,
                line_content: "Test again".to_string(),
                matches: vec![(0, 4)],
            },
        ];

        let flags = Flags {
            count: OptionState::Enabled,
            show_names: OptionState::Enabled,
            show_lines: OptionState::Disabled,
            ..Default::default()
        };

        print_count_results(&results, &flags);
    }

    #[test]
    fn test_print_count_results_without_file_names() {
        let results = vec![
            SearchMatch {
                file: "file1.txt".to_string(),
                line_number: 1,
                line_content: "This is a test".to_string(),
                matches: vec![(0, 4)],
            },
            SearchMatch {
                file: "file2.txt".to_string(),
                line_number: 2,
                line_content: "Another test".to_string(),
                matches: vec![(8, 12)],
            },
            SearchMatch {
                file: "file1.txt".to_string(),
                line_number: 3,
                line_content: "Test again".to_string(),
                matches: vec![(0, 4)],
            },
        ];

        let flags = Flags {
            count: OptionState::Enabled,
            show_names: OptionState::Disabled,
            show_lines: OptionState::Disabled,
            ..Default::default()
        };

        print_count_results(&results, &flags);
    }

    #[test]
    fn test_print_match_results_with_file_names_and_lines() {
        let results = vec![
            SearchMatch {
                file: "file1.txt".to_string(),
                line_number: 1,
                line_content: "This is a test".to_string(),
                matches: vec![(0, 4)],
            },
            SearchMatch {
                file: "file2.txt".to_string(),
                line_number: 2,
                line_content: "Another test".to_string(),
                matches: vec![(8, 12)],
            },
        ];

        let flags = Flags {
            count: OptionState::Disabled,
            show_names: OptionState::Enabled,
            show_lines: OptionState::Enabled,
            ..Default::default()
        };

        print_match_results(&results, &flags);
    }

    #[test]
    fn test_print_match_results_without_file_names_and_lines() {
        let results = vec![
            SearchMatch {
                file: "file1.txt".to_string(),
                line_number: 1,
                line_content: "This is a test".to_string(),
                matches: vec![(0, 4)],
            },
            SearchMatch {
                file: "file2.txt".to_string(),
                line_number: 2,
                line_content: "Another test".to_string(),
                matches: vec![(8, 12)],
            },
        ];

        let flags = Flags {
            count: OptionState::Disabled,
            show_names: OptionState::Disabled,
            show_lines: OptionState::Disabled,
            ..Default::default()
        };

        print_match_results(&results, &flags);
    }
}
