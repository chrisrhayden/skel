use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use serde::Deserialize;

use crate::{collect_project_config, Project};

use super::my_utils::get_home_dir;
use super::NewArgs;

pub type NewResult<T> = Result<T, Box<dyn Error>>;
pub type UserResult = NewResult<UserConfig>;

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub projects: HashMap<String, String>,
    pub alias: HashMap<String, Vec<String>>,
}

fn template_user_config(user_str: &str, old_string: &str) -> String {
    old_string.replace("{{new-config}}", user_str)
}

fn project_path_with_templateing(
    args: &NewArgs,
    user_config: &UserConfig,
    config_dir: &Option<String>,
) -> NewResult<PathBuf> {
    let project_pathbuf = if let Some(proj_str) = args.type_str.as_ref() {
        let p_string = find_project_file(user_config, &proj_str)?;

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

fn root_string_default_or(root: &Option<String>) -> String {
    if let Some(root) = root {
        root.to_owned()
    } else {
        // default to current_dir
        env::current_dir()
            .expect("cant get current_dir")
            .to_str()
            .expect("cant convet current dir to string")
            .to_owned()
    }
}

fn config_string_default_or(
    config_path: &Option<String>,
) -> (String, Option<String>) {
    if let Some(config_path) = config_path {
        (config_path.to_owned(), None)
    } else {
        let mut config_dir = get_home_dir();
        config_dir.push(".config");
        config_dir.push("new");
        let mut config_path = config_dir.clone();
        config_path.push("new_config.toml");

        (
            config_path
                .to_str()
                .expect("cant make str from default config  file")
                .to_owned(),
            Some(
                config_dir
                    .to_str()
                    .expect("cant make str from default config  file")
                    .to_owned(),
            ),
        )
    }
}

fn get_user_config(config_str: &str) -> UserResult {
    use std::io::Read;

    let config_path = PathBuf::from(config_str);

    if !config_path.exists() {
        return Err(Box::from(format!(
            "config dose not exists -- {}",
            config_path.to_str().expect("cant get root path"),
        )));
    }

    // TODO: more gracefully hand this
    let mut conf_file = fs::File::open(&config_path)
        .expect(&format!("can open file config path {:?}", config_path));

    let mut config_str = String::new();

    conf_file
        .read_to_string(&mut config_str)
        .expect("cant read to string");

    // TODO: let the user know whats wrong in a nice way
    let toml_conf = toml::from_str::<UserConfig>(&config_str)
        .expect(&format!("TOML Error -- {}", config_str));

    Ok(toml_conf)
}

pub fn config_str_to_user_struct(
    config_path: &Option<String>,
) -> NewResult<(UserConfig, Option<String>)> {
    let (u_s, u_d) = config_string_default_or(config_path);

    let conf = get_user_config(&u_s)?;

    Ok((conf, u_d))
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

// last takes precedents:
//      default > config > cli config
pub fn resolve_default(
    args: &NewArgs,
    user_config: &UserConfig,
    config_dir: &Option<String>,
) -> NewResult<Project> {
    // if we dont have a name here we never will
    let name = match args.name.clone() {
        None => return Err(Box::from(String::from("did not get name"))),
        Some(val) => val,
    };

    let project_pathbuf =
        project_path_with_templateing(&args, user_config, config_dir)?;

    // return if project path dose not exists, we will be able to make projects
    // from directory's eventually
    if !project_pathbuf.exists() {
        return Err(Box::from(format!(
            "project path given dos not exists -- {}",
            project_pathbuf.to_str().unwrap()
        )));
    }

    let project_config = collect_project_config(&project_pathbuf)?;

    let mut root_string = root_string_default_or(&args.root);
    // set root to the project name not the current_dir
    // or the one given on the cli
    // TODO: make this generic for windows maybe
    root_string.push('/');
    root_string.push_str(&name);

    Ok(Project::new(root_string, name, project_config))
}
