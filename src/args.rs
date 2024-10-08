use clap::{Parser, ValueEnum};

#[derive(Debug, PartialOrd, PartialEq, ValueEnum, Clone)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Parser, Debug)]
#[command(name = "getme")]
#[command(version = "1.0")]
#[command(author = "Your Name")]
#[command(about = "A Rust implementation of grep with parallel execution support")]
pub struct Config {
    #[arg(help = "The regex pattern to search for")]
    pub pattern: String,

    #[arg(help = "File or directory to search.")]
    pub files: Vec<String>,

    #[arg(short, long, help = "Enable parallel execution")]
    pub parallel: bool,

    #[arg(short, long, default_value = "quiet", help = "Verbosity level (quiet, normal, verbose)")]
    pub verbosity: Verbosity,

    #[arg(short, long, help = "Search all files in all subdirectories")]
    pub recursive: bool,

    #[arg(short, long, help = "Count the number of matches")]
    pub count: bool,
}

pub fn parse_args() -> Config {
    Config::parse()
}
