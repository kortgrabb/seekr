use clap::{Arg, Command};

use super::flag::Flags;

#[derive(Debug)]
pub struct Cli {
    pub pattern: String,
    pub files: Vec<String>,
}

pub fn parse_args() -> (Cli, Flags) {
    let matches = Command::new("getme")
        .version("1.0")
        .author("Your Name")
        .about("A Rust implementation of grep with parallel execution support")
        .arg(
            Arg::new("pattern")
                .help("The regex pattern to search for")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("files")
                .help("Files or directories to search")
                .required(true)
                .action(clap::ArgAction::Append)
                .index(2),
        )
        .args(Flags::args()) // Include flags from the Flags struct.
        .get_matches();

    // Extract pattern and files from matches.
    let pattern = matches.get_one::<String>("pattern").unwrap().clone();

    let files: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    // Extract flags from matches.
    let flags = Flags::from_matches(&matches);

    (Cli { pattern, files }, flags)
}
