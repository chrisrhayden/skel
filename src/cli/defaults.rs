use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use serde::Deserialize;

use crate::{collect_project_config, template::template, Project};

use super::parse_args::SkelArgs;

pub type SkelResult<T> = Result<T, Box<dyn Error>>;
pub type UserConfigResult = SkelResult<UserConfig>;

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub projects: HashMap<String, String>,
    pub alias: HashMap<String, Vec<String>>,
}

fn find_project_file(
    user_config: &UserConfig,
    type_str: &str,
) -> SkelResult<String> {
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

fn project_path_with_templateing(
    type_str: &str,
    user_config: &UserConfig,
    config_dir: &str,
) -> SkelResult<PathBuf> {
    let p_string = find_project_file(user_config, type_str)?;

    // this is lame but the only place empty string are used
    let p_string = template("", "", &config_dir, &p_string);

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
            .expect("cant get str from current dir")
            .to_owned()
    }
}

// TODO: make the path delimiter and config name variables
fn config_string_default() -> (String, String) {
    let mut config_dir = env::var("HOME").expect("home not set");

    // first make the directory
    config_dir.push('/');
    config_dir.push_str(".config");
    config_dir.push('/');
    config_dir.push_str("skel");

    // then the actual file
    let mut config_file = config_dir.clone();
    config_file.push('/');
    config_file.push_str("config.toml");

    (config_file, config_dir)
}

fn make_config_from_toml(config_str: &str) -> UserConfigResult {
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

fn collect_user_config() -> SkelResult<(UserConfig, String)> {
    let (u_s, u_d) = config_string_default();

    let conf = make_config_from_toml(&u_s)?;

    Ok((conf, u_d))
}

// last takes precedent:
//      default > config > cli config
pub fn resolve_default(args: SkelArgs) -> SkelResult<Project> {
    let (project_pathbuf, project_dir_str) = match &args.cli_project_file {
        Some(project_file) => {
            let config_dir_path = env::var("HOME").expect("HOME is not set");

            (PathBuf::from(project_file), config_dir_path)
        }
        None => {
            let (user_config, config_dir_path) = collect_user_config()?;

            (
                project_path_with_templateing(
                    &args.type_str,
                    &user_config,
                    &config_dir_path,
                )?,
                config_dir_path,
            )
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
    root_string.push_str(&args.name);

    Ok(Project::new(
        root_string,
        project_dir_str,
        &args,
        project_config,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::{make_fake_user_config, TempSetup};

    #[test]
    fn test_config_string_default_or_default() {
        let (test_config_path, test_config_dir) = config_string_default();

        let mut config_dir = env::var("HOME").expect("HOME not set");

        // first make the directory
        config_dir.push('/');
        config_dir.push_str(".config");
        config_dir.push('/');
        config_dir.push_str("skel");

        // then the actual file
        let mut config_file = config_dir.clone();
        config_file.push('/');
        config_file.push_str("config.toml");

        assert_eq!(
            test_config_dir, config_dir,
            "config_string_default_or did not make dir right"
        );

        assert_eq!(
            test_config_path, config_file,
            "did not make config_dir_path right"
        );
    }

    #[test]
    fn test_find_project_file() {
        let config = make_fake_user_config();

        let project = find_project_file(&config, "cp");

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_cpp.toml"),
            "failed to find project to make"
        );

        let project = find_project_file(&config, "p");

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_python.toml"),
            "failed to find project to make"
        );

        let project = find_project_file(&config, "basic_javascript");

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_javascript.toml"),
            "failed to find project to make"
        );
    }

    #[test]
    fn test_project_path_with_templateing() {
        let fake_config_dir = String::from("/tmp/fake_skel");

        let conf = make_fake_user_config();

        let project_path = match project_path_with_templateing(
            "cp",
            &conf,
            &fake_config_dir,
        ) {
            Err(err) => {
                assert!(false, "{}", err);
                // or just return
                unreachable!();
            }
            Ok(val) => val,
        };

        assert_eq!(
            project_path,
            PathBuf::from("/tmp/fake_skel/projects/basic_cpp.toml"),
            "failed to template path"
        );
    }

    #[test]
    fn test_make_config_from_toml() {
        let mut temp = TempSetup::default();
        let root = temp.setup();

        let mut temp_config = root.clone();

        temp_config.push("fake_skel");
        temp_config.push("fake_config.toml");

        temp.make_fake_user_config().expect("cant make user config");

        let user_config = match make_config_from_toml(
            temp_config.to_str().expect("cant get config path"),
        ) {
            Err(err) => {
                assert!(false, "{}", err);
                unreachable!();
            }
            Ok(val) => val,
        };

        let mut projects: HashMap<String, String> = HashMap::new();

        projects.insert(
            String::from("basic_python"),
            String::from("{{config-dir}}/projects/basic_python.toml"),
        );
        projects.insert(
            String::from("basic_cpp"),
            String::from("{{config-dir}}/projects/basic_cpp.toml"),
        );
        projects.insert(
            String::from("basic_javascript"),
            String::from("{{config-dir}}/projects/basic_javascript.toml"),
        );

        assert_eq!(
            user_config.projects, projects,
            "failsed to make user config projects"
        );

        let mut alias: HashMap<String, Vec<String>> = HashMap::new();

        alias.insert(
            String::from("basic_cpp"),
            vec![String::from("cpp"), String::from("cp"), String::from("c++")],
        );

        alias.insert(
            String::from("basic_python"),
            vec![String::from("py"), String::from("p")],
        );

        alias.insert(
            String::from("basic_javascript"),
            vec![String::from("js"), String::from("j")],
        );

        assert_eq!(
            user_config.alias, alias,
            "failed to make user config alias's"
        );
    }
}
