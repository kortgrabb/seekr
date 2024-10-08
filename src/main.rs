mod args;
mod search;

use args::parse_args;
use std::time::Instant;

fn main() {
    // Parse command line arguments to get the configuration.
    let config = parse_args();

    // Start the timer to measure execution time.
    let start = Instant::now();

    // Execute the search based on whether parallel processing is enabled.
    search::search_files(&config);

    // Calculate and print the duration of the search.
    let duration = start.elapsed();
    if config.verbosity == args::Verbosity::Verbose {
        println!("\nTime taken: {:?}", duration);
    }
}
