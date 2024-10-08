use app::cli::parse_args;
use search::{matcher::search_files, printer::print_results};

mod app;
mod search;

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
