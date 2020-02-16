// this is so we can use Project
mod fs_tools;
mod new_rs_error;
mod project;
mod system_tools;
mod template;
mod test_utils;

use std::error::Error;

pub use crate::project::{collect_config, FileTemplate, Project};

use crate::{fs_tools::make_project_tree, system_tools::call_build_script};

// the libs interface
pub fn make_project(project: &Project) -> Result<(), Box<dyn Error>> {
    // first make the project tree
    if let Err(err) = make_project_tree(project) {
        return Err(Box::from(err.to_string()));
    };

    // then try and run a build script
    if project.build.is_some() {
        call_build_script(project)
    } else {
        Ok(())
    }

    // TODO: maybe a quick test to see of it worked

    // done
}
