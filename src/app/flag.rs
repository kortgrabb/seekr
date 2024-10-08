use clap::{Arg, ArgAction, ArgMatches};

#[derive(Debug)]
pub struct Flags {
    pub parallel: bool,
    pub recursive: bool,
    pub count: bool,
    pub show_lines: bool,
    pub show_names: bool,
    pub ignore_case: bool,
    pub invert_match: bool,
}

impl Flags {
    // Define the flags used in the command line.
    pub fn args() -> Vec<Arg> {
        vec![
            Arg::new("parallel")
                .short('p')
                .long("parallel")
                .help("Enable parallel execution")
                .action(ArgAction::SetTrue),
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Search all files in all subdirectories")
                .action(ArgAction::SetTrue),
            Arg::new("count")
                .short('c')
                .long("count")
                .help("Only show the number of matches")
                .action(ArgAction::SetTrue),
            Arg::new("lines")
                .short('l')
                .long("lines")
                .help("Add line numbers to output")
                .action(ArgAction::SetTrue),
            Arg::new("ignore-case")
                .short('i')
                .long("ignore-case")
                .help("Ignore case when searching")
                .action(ArgAction::SetTrue),
            Arg::new("names")
                .short('n')
                .long("names")
                .help("Only show file names with matches")
                .action(ArgAction::SetTrue),
            Arg::new("invert-match")
                .short('v')
                .long("invert-match")
                .help("Matches all lines that do not contain the pattern")
                .action(ArgAction::SetTrue),
        ]
    }

    // Create the Flags object from command line matches.
    pub fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            parallel: *matches.get_one::<bool>("parallel").unwrap_or(&false),
            recursive: *matches.get_one::<bool>("recursive").unwrap_or(&false),
            count: *matches.get_one::<bool>("count").unwrap_or(&false),
            show_lines: *matches.get_one::<bool>("lines").unwrap_or(&false),
            ignore_case: *matches.get_one::<bool>("ignore-case").unwrap_or(&false),
            show_names: *matches.get_one::<bool>("names").unwrap_or(&false),
            invert_match: *matches.get_one::<bool>("invert-match").unwrap_or(&false),
        }
    }
}
