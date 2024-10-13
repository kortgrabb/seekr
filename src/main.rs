use app::cli::parse_args;
use search::file_io::get_all_files;
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
    get_all_files(needle, files, &flags)?;

    Ok(ExitCode::from(0))
}
