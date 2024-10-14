use app::args::{parse_args, Args};
use app::flags::Flags;
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
    let args = parse_args();

    // Determine if multi-threaded search is needed based on flags.
    let result = if args.flags.sequential.is_enabled() {
        search_with(args, search_files)?
    } else {
        search_with(args, search_files_parallel)?
    };

    // Check if any matches were found.
    if result.has_match() {
        Ok(ExitCode::from(0)) // Matches found
    } else {
        Ok(ExitCode::from(1)) // No matches found
    }

    // TODO: add modes
}

// Higher-level function to orchestrate search
fn search_with<F>(args: Args, search_fn: F) -> Result<SearchResult, Box<dyn std::error::Error>>
where
    F: Fn(
        &str,
        &[String],
        &Flags,
        &ignore::WalkBuilder,
        &AtomicBool,
    ) -> Result<SearchResult, Box<dyn std::error::Error>>,
{
    let matched = AtomicBool::new(false);
    let result = search_fn(
        &args.needle,
        &args.paths,
        &args.flags,
        &args.walk_builder(),
        &matched,
    )?;

    Ok(result)
}
