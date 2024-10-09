use clap::{Arg, ArgAction, ArgMatches};

#[derive(Debug, Default)]
pub struct Flags {
    pub parallel: bool,
    pub recursive: bool,
    pub count: bool,
    pub show_lines: bool,
    pub show_names: bool,
    pub ignore_case: bool,
    pub invert_match: bool,
    pub hidden: bool,
}

macro_rules! flag {
    ($name:literal, $short:literal, $long:literal, $help:expr) => {
        Arg::new($name)
            .short($short)
            .long($long)
            .help($help)
            .action(ArgAction::SetTrue)
    };
}

impl Flags {
    pub fn args() -> Vec<Arg> {
        vec![
            flag!("parallel", 'p', "parallel", "Enable parallel execution"),
            flag!(
                "recursive",
                'r',
                "recursive",
                "Search all files in all subdirectories"
            ),
            flag!("count", 'c', "count", "Only show the number of matches"),
            flag!("lines", 'n', "lines", "Add line numbers to output"),
            flag!(
                "ignore-case",
                'i',
                "ignore-case",
                "Ignore case when searching"
            ),
            flag!("names", 'l', "names", "Only show file names with matches"),
            flag!(
                "invert-match",
                'v',
                "invert-match",
                "Matches all lines that do not contain the pattern"
            ),
            flag!("hidden", 'H', "hidden", "Include hidden files in search"),
        ]
    }

    pub fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            parallel: matches.get_flag("parallel"),
            recursive: matches.get_flag("recursive"),
            count: matches.get_flag("count"),
            show_lines: matches.get_flag("lines"),
            ignore_case: matches.get_flag("ignore-case"),
            show_names: matches.get_flag("names"),
            invert_match: matches.get_flag("invert-match"),
            hidden: matches.get_flag("hidden"),
        }
    }
}
