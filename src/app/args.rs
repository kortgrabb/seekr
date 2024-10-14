use clap::{Arg, Command};

use super::flags::Flags;

#[derive(Debug)]
pub struct Cli {
    pub needle: String,
    pub paths: Vec<String>,
    pub _lua_script: Option<String>, // TODO: Add support for Lua scripts
    pub flags: Flags,
}

impl Cli {
    pub fn walk_builder(&self) -> ignore::WalkBuilder {
        let mut builder = ignore::WalkBuilder::new(&self.paths[0]);
        for path in self.paths.iter().skip(1) {
            builder.add(path);
        }

        builder
            .max_depth(self.flags.max_depth)
            .hidden(!self.flags.hidden.is_enabled());

        builder
    }
}

pub fn parse_args() -> Cli {
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
    let needle = matches
        .get_one::<String>("needle")
        .map(std::string::ToString::to_string)
        .unwrap();

    let files = matches
        .get_many::<String>("files")
        .map(|values| values.map(|v| v.to_string()).collect())
        .unwrap_or(vec![".".to_string()]);

    // TODO: add pipeline support

    let lua_script = matches
        .get_one::<String>("lua_script")
        .map(std::string::ToString::to_string);

    // Extract flags from matches.
    let flags = Flags::from_matches(&matches);

    Cli {
        needle,
        paths: files,
        _lua_script: lua_script,
        flags,
    }
}
