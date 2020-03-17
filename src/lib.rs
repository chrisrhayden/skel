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
    // first make the project tree
    make_project_tree(project).map_err(|err| err.into_string())?;

    // then try and run a build script
    if !project.dont_run_build && project.build.is_some() {
        call_build_script(project)
    } else {
        Ok(())
    }

    // TODO: maybe a quick test to see of it worked
}
