mod config;
mod parse_args;
mod project_tree;
mod templating;

use std::process;

// the real `main()` so we can clean up before `process::exit()`
fn run() -> bool {
    let args = parse_args::make_args();

    let config = match config::resolve_config(&args) {
        Err(err) => {
            eprintln!("{}", err);
            return false;
        }
        Ok(config) => config,
    };

    match project_tree::make_project_tree(&args, &config) {
        Err(err) => {
            eprintln!("{}", err);
            false
        }
        Ok(_) => true,
    }
}

/// this wraps `run()` so everything can be cleaned up before exiting with an error
/// code
fn main() {
    if !run() {
        process::exit(1);
    }
}
