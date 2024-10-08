use clap::{Arg, Command};

use super::flag::Flags;

#[derive(Debug)]
pub struct Cli {
    pub needle: String,
    pub files: Vec<String>,
}

pub fn parse_args() -> (Cli, Flags) {
    let matches = Command::new("getme")
        .version("0.1.0")
        .author("kortgrabb")
        .about("A Rust implementation of grep with parallel execution support")
        .arg(
            Arg::new("needle")
                .help("The regex pattern to search for in the haystack")
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
    let needle = match matches.get_one::<String>("needle") {
        Some(needle) => needle.to_string(),
        None => panic!("No needle provided"),
    };

    let files = matches
        .get_many::<String>("files")
        .map(|values| values.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // Extract flags from matches.
    let flags = Flags::from_matches(&matches);

    (Cli { needle, files }, flags)
}
