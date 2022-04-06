use std::{
    env::current_dir, error::Error, fs::metadata, path::PathBuf, process,
};

use skel::{
    config::resolve_config,
    parse_args::{make_args, SkelArgs},
    project_tree::make_project_tree,
};

fn get_root(args: &SkelArgs) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(diff_root) = &args.different_root {
        let diff_root_path = PathBuf::from(diff_root);

        if diff_root_path.is_dir() {
            Ok(diff_root_path)
        } else {
            Err(Box::from(format!(
                "different root does not exists or is not a dir {}",
                diff_root
            )))
        }
    } else {
        let root_path = current_dir().expect("could not get current dir");

        let root_string = root_path
            .to_str()
            .expect("could not parse root string")
            .into();

        Ok(root_string)
    }
}

// the real `main()` so we can clean up before `process::exit()`
fn run() -> Result<(), Box<dyn Error>> {
    let args = make_args();

    let mut root_string = get_root(&args)?;

    root_string.push(&args.name);

    if !args.dry_run && metadata(&root_string).is_ok() {
        return Err(Box::from(format!(
            "project exists {}",
            root_string.as_os_str().to_str().unwrap()
        )));
    }

    let config = resolve_config(&args, root_string)?;

    make_project_tree(&args, &config)
}

/// this wraps `run()` so everything can be cleaned up before exiting with an error
/// code
fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);

        process::exit(1);
    }

    process::exit(0);
}
