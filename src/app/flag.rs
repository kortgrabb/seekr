use clap::{Arg, ArgAction, ArgMatches};

// allw more than 3 boolean flags clippy

#[derive(Debug, Default)]
pub struct Flags {
    pub count: OptionState,
    pub show_lines: OptionState,
    pub show_names: OptionState,
    pub ignore_case: OptionState,
    pub invert_match: OptionState,
    pub hidden: OptionState,
}

#[derive(Debug, Default)]
pub enum OptionState {
    #[default]
    Disabled,
    Enabled,
}

impl OptionState {
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        *self = if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
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
                "Invert match to select non-matching lines"
            ),
            flag!(
                "hidden",
                'H',
                "hidden",
                "Search hidden files and directories"
            ),
        ]
    }

    pub fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            count: if matches.get_flag("count") {
                OptionState::Enabled
            } else {
                OptionState::Disabled
            },
            show_lines: if matches.get_flag("lines") {
                OptionState::Enabled
            } else {
                OptionState::Disabled
            },
            ignore_case: if matches.get_flag("ignore-case") {
                OptionState::Enabled
            } else {
                OptionState::Disabled
            },
            show_names: if matches.get_flag("names") {
                OptionState::Enabled
            } else {
                OptionState::Disabled
            },
            invert_match: if matches.get_flag("invert-match") {
                OptionState::Enabled
            } else {
                OptionState::Disabled
            },
            hidden: if matches.get_flag("hidden") {
                OptionState::Enabled
            } else {
                OptionState::Disabled
            },
        }
    }
}
