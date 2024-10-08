mod app;
mod core;

use app::cli::parse_args;
use core::search::{print_results, search_files};

fn main() {
    // Parse command line arguments to get pattern, files, and flags.
    let (cli, flags) = parse_args();

    let pattern = &cli.pattern;
    let files = &cli.files;

    // Search for the pattern in the files.
    let results = search_files(pattern, files, &flags);

    // Print the search results.
    print_results(&results, &flags);
}
