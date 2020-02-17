pub mod defaults;
pub mod my_utils;
pub mod parse_args;

use std::{error::Error, path::PathBuf};

use parse_args::NewArgs;

use defaults::{UserConfig, UserResult};

pub type NewResult<T> = Result<T, Box<dyn Error>>;

fn template_user_config(user_str: &str, old_string: &str) -> String {
    old_string.replace("{{new-config}}", user_str)
}

fn find_project_file(
    user_config: &UserConfig,
    type_str: &str,
) -> NewResult<String> {
    let type_string = type_str.to_string();

    if let Some(path_string) = user_config.projects.get(&type_string) {
        return Ok(path_string.clone());
    }

    let mut project_string = String::new();
    for (project, alias) in user_config.alias.iter() {
        // println!("--->>>>> {:?} {:?}", project, alias);
        if alias.contains(&type_string) && project_string.is_empty() {
            project_string.push_str(project);
        }
    }

    if project_string.is_empty() {
        return Err(Box::from(format!(
            "given project type not in user config -- {}",
            type_str
        )));
    }

    match user_config.projects.get(&project_string) {
        Some(val) => Ok(val.to_string()),
        None => {
            // this seams unlikely
            Err(Box::from(format!(
                "no project for that ailas -- {}",
                type_string
            )))
        }
    }
}

fn project_path_with_templateing(
    args: &NewArgs,
    user_config: &UserResult,
    config_dir: &Option<String>,
) -> NewResult<PathBuf> {
    let project_pathbuf = if let Some(proj_str) = args.type_str.as_ref() {
        if args.type_user_config && user_config.is_err() {
            if let Err(err) = user_config {
                return Err(Box::from(format!("{}", err)));
            } else {
                unreachable!();
            }
        } else if args.type_user_config && user_config.is_ok() {
            let p_string =
                find_project_file(user_config.as_ref().unwrap(), &proj_str)?;

            let p_string = if let Some(c_dir) = config_dir {
                template_user_config(c_dir, &p_string)
            } else {
                panic!("did not get project_dir string");
            };

            PathBuf::from(p_string)
        } else {
            PathBuf::from(proj_str)
        }
    } else {
        // we should not reach this
        panic!("did not receive the project type or file");
    };

    Ok(project_pathbuf)
}
