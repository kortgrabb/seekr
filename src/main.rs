use app::args::{parse_args, Args};
use search::searcher::{search_files, search_files_parallel, SearchResult};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

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
    let args = parse_args();
    let flags = &args.flags;

    let needle = &args.needle;
    let files = &args.paths;

    let matched = AtomicBool::new(false);

    // Determine if multi-threaded search is needed based on flags.
    let result = if flags.sequential.is_enabled() {
        search(needle, files, &args, &matched)?
    } else {
        search_parallel(needle, files, &args, &matched)?
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
    args: &Args,
    matched: &AtomicBool,
) -> Result<SearchResult, Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();

    let walker = args.walk_builder();
    let result = search_files(needle, files, &args.flags, &walker, matched)?;
    let elapsed = started.elapsed();

    println!(
        "Execution time: {}s {}ms",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    Ok(result)
}

// Higher-level function to orchestrate multi-threaded search
fn search_parallel(
    needle: &str,
    files: &[String],
    args: &Args,
    matched: &AtomicBool,
) -> Result<SearchResult, Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();

    let walker = args.walk_builder();
    let result = search_files_parallel(needle, files, &args.flags, &walker, matched)?;
    let elapsed = started.elapsed();

    println!(
        "Execution time: {}s {}ms",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    Ok(result)
}
