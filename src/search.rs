use crate::args::{Config, Verbosity};
use colored::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead};
use regex::Regex;
use walkdir::WalkDir;

// Function to search files based on configuration.
pub fn search_files(cfg: &Config) {
    let files = get_all_files(cfg);

    let search_fn = |file: &String| {
        if let Err(err) = search_file(file, &cfg.pattern, cfg) {
            eprintln!("Error searching file {}: {}", file, err);
        }
    };

    if cfg.parallel {
        files.par_iter().for_each(search_fn);
    } else {
        files.iter().for_each(search_fn);
    }
}

// Function to search for the pattern in a single file.
fn search_file(file: &str, pattern: &str, cfg: &Config) -> io::Result<()> {
    let file_handle = File::open(file)?;
    let reader = io::BufReader::new(file_handle);
    let mut count = 0;

    let regex = Regex::new(pattern).unwrap();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let num_matches = regex.find_iter(&line).count();

        if num_matches > 0 {
            // If the count flag is set
            if cfg.count {
                count += num_matches;
            }

            let highlighted = regex.replace_all(&line, |m: &regex::Captures| {
                format!("{}", m.get(0).unwrap().as_str().cyan())
            });

            match cfg.verbosity {
                Verbosity::Quiet => println!("{}", line),
                Verbosity::Normal => println!("{}: {}", line_number + 1, highlighted),
                Verbosity::Verbose => println!("{}:{}: {}", file, line_number + 1, highlighted),
            }
        }
    }

    if cfg.count {
        println!(); // Add a newline before printing the count.
        println!("matches in {}: {}", file, count);
    }

    Ok(())
}

// Function to get all files from the provided path (file or directory).
fn get_all_files(cfg: &Config) -> Vec<String> {
    let mut files = Vec::new();

    for path_name in &cfg.files {
        let path = std::path::Path::new(path_name);
        if path.is_dir() {
            let walker = WalkDir::new(path).into_iter();
            for entry in walker.filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && (cfg.recursive || entry.depth() == 1) {
                    files.push(entry.path().to_str().unwrap().to_string());
                }
            }
        } else if path.is_file() {
            files.push(path_name.clone());
        } else {
            eprintln!("Warning: '{}' is neither a file nor a directory.", path_name);
        }
    }

    files
}