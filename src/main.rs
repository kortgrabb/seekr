use app::cli::parse_args;
use app::flag::Flags;
use search::searcher::{search_files, search_files_parallel, SearchResult};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

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

    let matched = AtomicBool::new(false);

    // Determine if multi-threaded search is needed based on flags.
    let result = if flags.sequential.is_enabled() {
        search(needle, files, &flags, &matched)?
    } else {
        search_parallel(needle, files, &flags, &matched)?
    };

    // Check if any matches were found.
    if result.has_match() {
        Ok(ExitCode::from(0)) // Matches found
    } else {
        Ok(ExitCode::from(1)) // No matches found
    }
}

// Higher-level function to orchestrate single-threaded search
fn search(
    needle: &str,
    files: &[String],
    flags: &Flags,
    matched: &AtomicBool,
) -> Result<SearchResult, Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();

    let result = search_files(needle, files, flags, matched)?;

    let _ = started.elapsed(); // TODO

    Ok(result)
}

// Higher-level function to orchestrate multi-threaded search
fn search_parallel(
    needle: &str,
    files: &[String],
    flags: &Flags,
    matched: &AtomicBool,
) -> Result<SearchResult, Box<dyn std::error::Error>> {
    let result = search_files_parallel(needle, files, flags, matched)?;
    Ok(result)
}
