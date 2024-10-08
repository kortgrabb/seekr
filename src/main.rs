mod app;
mod core;

use core::search::{print_results, search_files};

use app::cli::parse_args;

fn main() {
    // Parse command line arguments to get pattern, files, and flags.
    let (cli, flags) = parse_args();

    let needle = &cli.needle;
    let files = &cli.files;

    // Search for the needle in the files.
    let results = search_files(needle, files, &flags);
    // Print the search results.
    match results {
        Ok(results) => print_results(&results, &flags),
        Err(e) => eprintln!("Error: {}", e),
    }
}
