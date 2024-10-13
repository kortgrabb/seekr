use app::cli::parse_args;
use plugin_integration::lua_plugin::LuaPlugin;
use rlua::{Lua, Result as LuaResult, RluaCompat, Value};
use search::{
    matcher::search_files,
    printer::{print_results, print_with_lua_callback},
};
use std::process::ExitCode;

mod app;
mod plugin_integration;

mod search;
/* Exit codes:
 * 0 - Matches found
 * 1 - No matches found
 * 2 - Error during execution
 */

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    // Parse command line arguments to get pattern, files, and flags.
    let (cli, flags) = parse_args();

    let needle = &cli.needle;
    let files = &cli.files;

    // Search for the needle in the files.
    let results = search_files(needle, files, &flags)?;

    // If there is a Lua script provided in the CLI, evaluate it.
    if let Some(script_path) = &cli.lua_script {
        let lua_plugin = LuaPlugin::new();
        lua_plugin.run_script(script_path, &results)?;
        // If there is a callback function in the Lua script, execute it.
        let callback_name = "process_result";
        if lua_plugin.has_function(callback_name)? {
            lua_plugin.execute_callback(callback_name, &results)?;
        } else {
            print_results(&results, &flags);
        }
    }
    Ok(ExitCode::from(0))
}
