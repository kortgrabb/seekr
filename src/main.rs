use app::cli::parse_args;
use plugin_integration::lua_plugin::LuaPlugin;
use rlua::{Lua, Result as LuaResult, RluaCompat, Value};
use search::{
    file_io::get_all_files,
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

    let files = get_all_files(needle, files, &flags)?;

    // // If there is a Lua script provided in the CLI, evaluate it.
    // if let Some(script_path) = &cli.lua_script {
    //     let lua_plugin = LuaPlugin::new();

    //     // If there is a callback function in the Lua script, execute it.
    //     lua_plugin.run_script(script_path, &results)?;

    //     let callback_name: &str = "process_result";
    //     if lua_plugin.has_function(callback_name)? {
    //         print_with_lua_callback(&results, &flags, &lua_plugin, callback_name)?;
    //     }
    // }
    Ok(ExitCode::from(0))
}
