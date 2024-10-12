use app::cli::parse_args;
use search::{matcher::search_files, printer::print_results};
use std::process::ExitCode;

mod app;
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

    // Handle the search results.
    if results.is_empty() {
        // No matches found
        Ok(ExitCode::from(1))
    } else {
        // Print the search results.
        print_results(&results, &flags);
        Ok(ExitCode::from(0))
    }
}
