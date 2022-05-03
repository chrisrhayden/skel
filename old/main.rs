/// make project layout form a config file
mod args;
mod config;
mod fs_tools;
mod process_tools;
mod skeleton;
mod template;
mod test_utils;

use std::{error::Error, fs::metadata};

use crate::{fs_tools::make_project_tree, process_tools::call_build_script};

pub use crate::skeleton::{Skeleton, SkeletonConfig};

/// make a new project from a Project struct
pub fn make_project(project: &Skeleton) -> Result<(), Box<dyn Error>> {
    if metadata(&project.project_root_string).is_ok() {
        // if metadata succeeds then the file exists
        return Err(Box::from(format!(
            "project destination exists -- {}",
            project.project_root_string,
        )));
    }

    // this isn't the worst
    if project.build_first && !project.dont_run_build {
        call_build_script(project)?;
    }

    // make the project tree
    make_project_tree(project)?;

    // then try and run a build script if it wasn't run before
    if !project.build_first && !project.dont_run_build {
        call_build_script(project)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;

    let project = config::resolve_defaults(args)?;

    make_project(&project)
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{fs, path::PathBuf};

    use crate::test_utils::{make_fake_skeleton, TempSetup};

    #[test]
    fn test_make_project_root_exits() {
        let mut temp = TempSetup::default();
        let project_root_path: PathBuf = temp.setup();

        let proj =
            make_fake_skeleton(Some(project_root_path.to_str().unwrap()));

        let mut src = project_root_path;
        src.push("test_project");
        src.push("src");

        if let Err(err) = fs::create_dir_all(&src) {
            eprintln!("{}", err);
            panic!("something is fucked");
        }

        if let Err(err) = make_project(&proj) {
            let err_string = format!(
                "project destination exists -- {}",
                proj.project_root_string
            );

            assert_eq!(
                format!("{}", err),
                err_string,
                "did not find project path"
            );
        } else {
            panic!("did not fail");
        };
    }
}
