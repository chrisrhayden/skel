pub mod defaults;
pub mod my_utils;
pub mod parse_args;

use std::{error::Error, path::PathBuf};

use parse_args::NewArgs;

use defaults::{find_project_file, UserConfig};

pub type NewResult<T> = Result<T, Box<dyn Error>>;

fn template_user_config(user_str: &str, old_string: &str) -> String {
    old_string.replace("{{new-config}}", user_str)
}

fn project_path_with_templateing(
    args: &NewArgs,
    user_config: &UserConfig,
    config_dir: &Option<String>,
) -> NewResult<PathBuf> {
    let project_pathbuf = if let Some(proj_str) = args.type_str.as_ref() {
            let p_string =
                find_project_file(user_config, &proj_str)?;

            let p_string = if let Some(c_dir) = config_dir {
                template_user_config(c_dir, &p_string)
            } else {
                panic!("did not get project_dir string");
            };

            PathBuf::from(p_string)
    } else {
        // we should not reach this
        panic!("did not receive the project type or file");
    };

    Ok(project_pathbuf)
}
