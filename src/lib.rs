// this is so we can use Project
mod fs_tools;
mod new_rs_error;
mod project;
mod system_tools;
mod template;
mod test_utils;

use std::error::Error;

pub use crate::project::{Template, Project, collect_config};

use crate::{fs_tools::make_project_tree, system_tools::call_build_script};

// the libs interface
pub fn make_project(project: &Project) -> Result<(), Box<dyn Error>> {
    if let Err(err) = make_project_tree(project) {
        return Err(Box::from(err.to_string()));
    };

    call_build_script(project)
}
