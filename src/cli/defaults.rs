use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use serde::Deserialize;

use crate::{collect_project_config, Project};

use super::parse_args::NewArgs;

pub type NewResult<T> = Result<T, Box<dyn Error>>;
pub type UserResult = NewResult<UserConfig>;

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub projects: HashMap<String, String>,
    pub alias: HashMap<String, Vec<String>>,
}

fn find_project_file(
    user_config: UserConfig,
    type_str: String,
) -> NewResult<String> {
    let type_string = type_str.to_string();

    if let Some(path_string) = user_config.projects.get(&type_string) {
        return Ok(path_string.clone());
    }

    let mut project_string = String::new();

    for (project, alias) in user_config.alias.iter() {
        if alias.contains(&type_string) {
            project_string.clone_from(project);
            break;
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
            // this seems unlikely
            Err(Box::from(format!(
                "no project for that alias -- {}",
                type_string
            )))
        }
    }
}

fn template_user_config(user_str: &str, old_string: &str) -> String {
    old_string.replace("{{new-config}}", user_str)
}

fn project_path_with_templateing(
    type_str: String,
    user_config: UserConfig,
    config_dir: String,
) -> NewResult<PathBuf> {
    let p_string = find_project_file(user_config, type_str)?;

    let p_string = template_user_config(&config_dir, &p_string);

    Ok(PathBuf::from(p_string))
}

fn root_string_default_or(root_from_cli: &Option<String>) -> String {
    if let Some(from_cli) = root_from_cli {
        from_cli.to_owned()
    } else {
        // default to current_dir
        env::current_dir()
            .expect("cant get current_dir")
            .to_str()
            .expect("cant convet current dir to string")
            .to_owned()
    }
}

// TODO: make the path delimiter and config name variables
fn config_string_default_or(
    config_path_from_cli: Option<String>,
) -> (String, String) {
    let mut config_dir = match config_path_from_cli {
        Some(config_path) => config_path.clone(),
        None => env::var("HOME").expect("cant get env var HOME"),
    };

    // first make the directory
    config_dir.push_str(".config");
    config_dir.push('/');
    config_dir.push_str("new");

    // then the actual file
    let mut config_path = config_dir.clone();
    config_path.push_str("config.toml");

    (config_path, config_dir)
}

fn make_config_from_toml(config_str: &str) -> UserResult {
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

pub fn collect_user_config(
    config_path: Option<String>,
) -> NewResult<(UserConfig, String)> {
    let (u_s, u_d) = config_string_default_or(config_path);

    let conf = make_config_from_toml(&u_s)?;

    Ok((conf, u_d))
}

// last takes precedent:
//      default > config > cli config
pub fn resolve_default(args: NewArgs) -> NewResult<Project> {
    let name = args.name;

    let project_pathbuf = match args.cli_project_file {
        Some(project_file) => PathBuf::from(project_file),
        None => {
            let (user_config, config_dir_path) =
                collect_user_config(args.cli_config_path)?;

            project_path_with_templateing(
                args.type_str,
                user_config,
                config_dir_path,
            )?
        }
    };

    // return if project path dose not exists
    if !project_pathbuf.exists() {
        return Err(Box::from(format!(
            "project path given dos not exists -- {}",
            project_pathbuf.to_str().unwrap()
        )));
    }

    let project_config = collect_project_config(&project_pathbuf)?;

    let mut root_string = root_string_default_or(&args.different_root);
    // set root to the project name not the current_dir
    // or the one given on the cli
    // TODO: make this generic for windows maybe
    root_string.push('/');
    root_string.push_str(&name);

    Ok(Project::new(root_string, name, project_config))
}
