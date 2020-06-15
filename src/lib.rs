//! make project layout form a config file

pub mod cli;
mod fs_tools;
mod project;
mod skel_error;
mod system_tools;
mod template;
mod test_utils;

use std::error::Error;

use crate::{
    fs_tools::make_project_tree, project::collect_project_config,
    system_tools::call_build_script,
};

pub use crate::project::{
    Project, ProjectArgs, ProjectConfigFile, ProjectTemplate,
};

pub use crate::fs_tools::collect_string_from_file;

///! make a new project from a Project struct
pub fn make_project(project: &Project) -> Result<(), Box<dyn Error>> {
    // check if something exists at root, root being /path/to/project_name
    if project.project_root_path.exists() {
        return Err(Box::from(format!(
            "project destination exists -- {}",
            project.root_string()
        )));
    }

    // this isn't the worst
    if project.build_first && !project.dont_run_build && project.build.is_some()
    {
        call_build_script(project)?;
    }

    // first make the project tree
    make_project_tree(project).map_err(|err| err.into_string())?;

    // then try and run a build script
    if !project.build_first
        && !project.dont_run_build
        && project.build.is_some()
    {
        call_build_script(project)?;
    }

    Ok(())

    // TODO: maybe a quick test to see of it worked
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{fs, path::PathBuf};

    use crate::test_utils::{make_fake_project, TempSetup};

    #[test]
    fn test_make_project_root_exits() {
        let mut temp = TempSetup::default();
        let project_root_path: PathBuf = temp.setup();

        let proj = make_fake_project(Some(project_root_path.clone()));

        let mut src = project_root_path.clone();
        src.push("test_project");
        src.push("src");

        if let Err(err) = fs::create_dir_all(&src) {
            eprintln!("{}", err);
            assert!(false, "something is fucked");
        }

        if let Err(err) = make_project(&proj) {
            let err_string =
                format!("project destination exists -- {}", proj.root_string());

            assert_eq!(
                format!("{}", err),
                err_string,
                "did not find project path"
            );
        } else {
            assert!(false, "did not fail");
        };

        assert!(true);
    }
}
