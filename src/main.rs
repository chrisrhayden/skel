mod config;
mod parse_args;
mod templating;

use std::process;

// the real `main()` so we can clean up before `process::exit()`
fn run() -> bool {
    let args = parse_args::make_args();

    if let Err(err) = config::resolve_config(&args) {
        eprintln!("{}", err);

        false
    } else {
        true
    }
}

/// this wraps `run()` so everything can be cleaned up before exiting with an error
/// code
fn main() {
    if !run() {
        process::exit(1);
    }
}
