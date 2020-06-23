use std::{collections::HashMap, env, error::Error, path::PathBuf};

use serde::Deserialize;

use crate::{
    fs_tools::collect_string_from_file, template::template, Project,
    ProjectConfig,
};

use super::parse_args::SkelArgs;

pub type SkelResult<T> = Result<T, Box<dyn Error>>;

struct ConfigPaths {
    config_file: PathBuf,
    config_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub projects: HashMap<String, String>,
    pub alias: HashMap<String, Vec<String>>,
}

fn get_user_config(config_path: &PathBuf) -> SkelResult<UserConfig> {
    if !config_path.exists() {
        return Err(Box::from(format!(
            "config dose not exists -- {}",
            config_path.to_str().expect("cant get root path"),
        )));
    }

    let config_str = collect_string_from_file(&config_path)?;

    // TODO: let the user know whats wrong in a nice way
    // idk, maybe just say bad config with the file name
    let toml_conf = toml::from_str::<UserConfig>(&config_str)
        .expect(&format!("TOML Error -- {}", config_str));

    Ok(toml_conf)
}

fn default_config_paths() -> ConfigPaths {
    let mut config_dir = env::var("HOME")
        .expect("HOME not set")
        .parse::<PathBuf>()
        .expect("cant make path buf from config string");

    // first make the directory
    config_dir.push(".config");
    config_dir.push("skel");

    // then the actual file
    let mut config_file = config_dir.clone();
    config_file.push("config.toml");

    ConfigPaths {
        config_file,
        config_dir,
    }
}

fn collect_user_config() -> SkelResult<(UserConfig, PathBuf)> {
    let config_paths = default_config_paths();

    let conf = get_user_config(&config_paths.config_file)?;

    Ok((conf, config_paths.config_dir))
}

fn find_project_file(
    user_config: &UserConfig,
    type_string: String,
) -> SkelResult<String> {
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
            type_string
        )));
    }

    match user_config.projects.get(&project_string) {
        Some(val) => Ok(val.to_string()),
        None => Err(Box::from(format!(
            "no project for that alias -- {}",
            type_string
        ))),
    }
}

fn project_path_with_templateing(
    type_str: String,
    user_config: &UserConfig,
    config_dir: &str,
) -> SkelResult<PathBuf> {
    let p_string = find_project_file(user_config, type_str)?;

    // this is lame but the only place empty strings are used
    let p_string = template("", "", &config_dir, &p_string);

    Ok(PathBuf::from(p_string))
}

fn resolve_type_config(
    mut type_string: Option<String>,
    mut project_file: Option<String>,
) -> SkelResult<(PathBuf, PathBuf)> {
    if let Some(project_file) = project_file.take() {
        let config_paths = default_config_paths();

        Ok((PathBuf::from(project_file), config_paths.config_dir))
    } else if let Some(type_string) = type_string.take() {
        let (user_config, config_dir_path) = collect_user_config()?;

        let project_config_path = project_path_with_templateing(
            type_string,
            &user_config,
            config_dir_path
                .to_str()
                .as_ref()
                .expect("cant get config path str"),
        )?;

        Ok((project_config_path, config_dir_path))
    } else {
        Err(Box::from("pleas give ether a project file or type str"))
    }
}

// return a config from a toml file
fn get_project_config(path: &PathBuf) -> Result<ProjectConfig, Box<dyn Error>> {
    if !path.exists() {
        return Err(Box::from(format!(
            "path given dose exists -- {}",
            path.to_str().unwrap()
        )));
    }

    let config_string = collect_string_from_file(&path)?;

    let config = toml::from_str::<ProjectConfig>(&config_string).expect(
        &format!("Toml Error in project file - {}", path.to_str().unwrap()),
    );

    Ok(config)
}

fn resolve_project_root(name: &str, root_from_cli: &Option<String>) -> String {
    let mut r_string = if let Some(from_cli) = root_from_cli {
        from_cli.to_owned()
    } else {
        // default to current_dir
        env::current_dir()
            .expect("cant get current_dir")
            .to_str()
            .expect("cant get str from current dir")
            .to_owned()
    };

    // set root to the project name not the current_dir
    // or the one given on the cli
    // TODO: make this generic for windows maybe
    r_string.push('/');
    r_string.push_str(name);

    r_string
}

// last takes precedent:
//      default > config > cli config
pub fn resolve_default(args: SkelArgs) -> SkelResult<Project> {
    let (config_pathbuf, config_dir_path) =
        resolve_type_config(args.type_str, args.cli_project_file)?;

    let root_string = resolve_project_root(&args.name, &args.different_root);

    let mut file_config = get_project_config(&config_pathbuf)?;

    file_config.resolve_project_templates(
        &root_string,
        &args.name,
        config_dir_path
            .to_str()
            .as_ref()
            .expect("cant unwrap config_dir_path"),
    )?;

    if file_config.files.is_none()
        && file_config.dirs.is_none()
        && file_config.build.is_none()
    {
        return Err(Box::from("project dose not have anything to do"));
    }

    let build_first = if args.build_first
        || (file_config.build_first.is_some()
            && file_config.build_first.unwrap())
    {
        true
    } else {
        false
    };

    let project = Project {
        build_first,
        dirs: file_config.dirs,
        files: file_config.files,
        build: file_config.build,
        templates: file_config.templates,
        config_dir_string: config_dir_path
            .to_str()
            .expect("cant get config_dir_path as str")
            .to_string(),
        name: args.name,
        project_root_path: PathBuf::from(&root_string),
        project_root_string: root_string,
        dont_make_template: args.dont_make_templates,
        dont_run_build: args.dont_run_build,
        show_build_output: args.show_build_output,
    };

    Ok(project)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::{
        make_fake_conifg_file, make_fake_user_config, TempSetup,
    };

    #[test]
    fn test_config_string_default_or_default() {
        let test_config_paths = default_config_paths();

        let mut config_dir = env::var("HOME")
            .expect("HOME not set")
            .parse::<PathBuf>()
            .unwrap();

        // first make the directory
        config_dir.push(".config");
        config_dir.push("skel");

        // then the actual file
        let mut config_file = config_dir.clone();
        config_file.push("config.toml");

        assert_eq!(
            test_config_paths.config_dir, config_dir,
            "config_string_default_or did not make dir right"
        );

        assert_eq!(
            test_config_paths.config_file, config_file,
            "did not make config_dir_path right"
        );
    }

    #[test]
    fn test_find_project_file() {
        let config = make_fake_user_config();

        let project = find_project_file(&config, "cp".to_string());

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_cpp.toml"),
            "failed to find project to make"
        );

        let project = find_project_file(&config, "p".to_string());

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_python.toml"),
            "failed to find project to make"
        );

        let project =
            find_project_file(&config, "basic_javascript".to_string());

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_javascript.toml"),
            "failed to find project to make"
        );
    }

    #[test]
    fn test_project_path_with_templateing() {
        let fake_config_dir = String::from("/tmp/skel");

        let conf = make_fake_user_config();

        let project_path = match project_path_with_templateing(
            "cp".to_string(),
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
            PathBuf::from("/tmp/skel/projects/basic_cpp.toml"),
            "failed to template path"
        );
    }

    #[test]
    fn test_make_config_from_toml() {
        let mut temp = TempSetup::default();
        let root = temp.setup();

        let mut temp_config = root.clone();

        temp_config.push(".config");
        temp_config.push("skel");
        temp_config.push("config.toml");

        temp.make_fake_user_config().expect("cant make user config");

        let user_config = match get_user_config(&temp_config) {
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

    #[test]
    fn test_resolve_user_config_no_user_path() {
        let mut temp = TempSetup::default();
        let root = temp.setup();

        let fake_home = root.to_str().unwrap();
        env::set_var("HOME", fake_home);

        temp.make_fake_user_config()
            .expect("did not make fake config");

        let type_str = Some("cpp".to_string());
        let user_path = None;

        let mut fake_config = fake_home.clone().parse::<PathBuf>().unwrap();

        fake_config.push(".config");
        fake_config.push("skel");

        let mut fake_config_file = fake_config.clone();

        fake_config_file.push("projects");
        fake_config_file.push("basic_cpp.toml");

        match resolve_type_config(type_str, user_path) {
            Ok((proj_path, proj_dir)) => {
                assert_eq!(
                    proj_path,
                    PathBuf::from(fake_config_file),
                    "did not find c++ toml file"
                );

                assert_eq!(
                    proj_dir, fake_config,
                    "did not make proj_dir right"
                );
            }
            Err(err) => assert!(false, "Error: {}", err),
        };
    }

    #[test]
    fn test_resolve_user_config_user_path_provided() {
        let type_str = Some("cpp".to_string());

        let user_path_str = String::from("/tmp/skel/projects/cpp.toml");

        env::set_var("HOME", "/home/test");

        match resolve_type_config(type_str, Some(user_path_str)) {
            Ok((proj_path, proj_dir)) => {
                assert_eq!(
                    proj_path,
                    PathBuf::from("/tmp/skel/projects/cpp.toml"),
                    "did not find c++ toml file"
                );

                assert_eq!(
                    proj_dir,
                    PathBuf::from("/home/test/.config/skel"),
                    "did not make proj_dir right"
                );
            }
            Err(err) => assert!(false, "Error: {}", err),
        };
    }

    #[test]
    fn test_collect_config() {
        let mut temp = TempSetup::default();
        let mut fake_path = temp.setup();

        fake_path.push("fake_project.toml");

        if !make_fake_conifg_file(&fake_path) {
            assert!(false, "failed to make fake config in temp dir");
        }

        match get_project_config(&fake_path) {
            Err(err) => assert!(false, "{} bad toml config", err),
            Ok(config) => {
                assert_eq!(
                    config.dirs.as_ref().unwrap()[0],
                    String::from("src"),
                    "did not get the right name"
                );

                for entry in config.dirs.as_ref().unwrap().iter() {
                    assert_eq!(entry.is_empty(), false, "no dirs in array");
                }
            }
        };
    }
}
