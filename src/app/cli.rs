use clap::{Arg, Command};

use super::flag::Flags;

#[derive(Debug)]
pub struct Cli {
    pub needle: String,
    pub files: Vec<String>,
    pub lua_script: Option<String>,
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
        .arg(
            Arg::new("lua_script")
                .long("lua")
                .value_name("SCRIPT")
                .help("Lua script to execute on search results"),
        )
        .args(Flags::args())
        .get_matches();

    // Extract pattern and files from matches.
    let needle = matches.get_one::<String>("needle").unwrap().to_string();

    let files = matches
        .get_many::<String>("files")
        .unwrap()
        .map(std::string::ToString::to_string)
        .collect();

    let lua_script = matches
        .get_one::<String>("lua_script")
        .map(std::string::ToString::to_string);

    // Extract flags from matches.
    let flags = Flags::from_matches(&matches);

    (
        Cli {
            needle,
            files,
            lua_script,
        },
        flags,
    )
}
