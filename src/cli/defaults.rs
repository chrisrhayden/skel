use std::{collections::HashMap, env, error::Error, ffi::OsStr, path::PathBuf};

use serde::Deserialize;

use crate::{
    fs_tools::collect_string_from_file,
    template::{template, TemplateArgs},
    Project, ProjectConfig,
};

use super::parse_args::SkelArgs;

pub type SkelResult<T> = Result<T, Box<dyn Error>>;

struct ConfigPaths {
    config_file: String,
    config_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub projects: HashMap<String, String>,
    pub alias: HashMap<String, Vec<String>>,
}

fn get_user_config<P>(config_path: &P) -> SkelResult<UserConfig>
where
    P: AsRef<OsStr> + std::fmt::Debug,
{
    let config_path: PathBuf = PathBuf::from(config_path);
    if !config_path.exists() {
        return Err(Box::from(format!(
            "config dose not exists -- {:?}",
            config_path,
        )));
    }

    let config_str = collect_string_from_file(&config_path)?;

    // TODO: let the user know whats wrong in a nice way
    // idk, maybe just say bad config with the file name
    let toml_conf = toml::from_str::<UserConfig>(&config_str)
        .expect(&format!("TOML Error -- {}", config_str));

    Ok(toml_conf)
}

fn default_skel_config_paths() -> ConfigPaths {
    let mut config_dir = env::var("HOME")
        .expect("HOME not set")
        .parse::<String>()
        .expect("cant make path buf from config string");

    // first make the directory
    config_dir.push_str("/.config");
    config_dir.push_str("/skel");

    // then the actual file
    let mut config_file = config_dir.clone();
    config_file.push_str("/config.toml");

    ConfigPaths {
        config_file,
        config_dir,
    }
}

fn find_project_file(
    user_config: &UserConfig,
    alias_string: String,
) -> SkelResult<String> {
    // if the alias string is an exact project key
    if let Some(path_string) = user_config.projects.get(&alias_string) {
        return Ok(path_string.clone());
    }

    // find the project key for the given alias
    // if found clone it to project_string
    for (project, alias) in user_config.alias.iter() {
        if alias.contains(&alias_string) {
            let project_config_path =
                user_config.projects.get(project).map(String::from);

            if let Some(config_path) = project_config_path {
                return Ok(config_path);
            } else {
                return Err(Box::from(format!(
                    "no project for alias -- {}",
                    alias_string
                )));
            }
        }
    }

    Err(Box::from(format!(
        "no given alias in user config -- {}",
        alias_string
    )))
}

fn project_path_with_templateing(
    alias_str: String,
    user_config: &UserConfig,
    config_dir: &str,
) -> SkelResult<String> {
    let p_string = find_project_file(user_config, alias_str)?;

    let template_args = TemplateArgs {
        root_path: "",
        project_name: "",
        skel_config_path: config_dir,
    };

    // this is lame but the only place empty strings are used
    Ok(template(&template_args, &p_string))
}

fn resolve_project_path(alias_string: String) -> SkelResult<(String, String)> {
    let config_paths = default_skel_config_paths();

    let user_config = get_user_config(&config_paths.config_file)?;

    let project_config_path = project_path_with_templateing(
        alias_string,
        &user_config,
        &config_paths.config_dir,
    )?;

    Ok((project_config_path, config_paths.config_dir))
}

// return a config from a toml file
fn get_project_config<P>(
    project_str: &P,
) -> Result<ProjectConfig, Box<dyn Error>>
where
    P: AsRef<OsStr> + std::fmt::Debug,
{
    let project_path = PathBuf::from(project_str);

    if !project_path.exists() {
        return Err(Box::from(format!(
            "path given dose exists -- {}",
            project_path.to_str().unwrap()
        )));
    }

    let config_string = collect_string_from_file(&project_path)?;

    let config = toml::from_str::<ProjectConfig>(&config_string)
        .expect(&format!("Toml Error in project file - {:?}", project_str));

    Ok(config)
}

fn resolve_project_root(name: &str, root_from_cli: Option<String>) -> String {
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
    r_string.push('/');
    r_string.push_str(name);

    r_string
}

// last takes precedent:
//      default > config > cli config
pub fn resolve_default(mut args: SkelArgs) -> SkelResult<Project> {
    let root_string = resolve_project_root(&args.name, args.different_root);

    // get the project config and the default skel dir path for templates
    let (project_path, skel_config_path) =
        if let Some(project_file) = args.cli_project_file.take() {
            let config_paths = default_skel_config_paths();

            (project_file, config_paths.config_dir)
        } else {
            resolve_project_path(args.alias_str)?
        };

    let mut project_config = get_project_config(&project_path)?;

    project_config.resolve_project_templates(
        &root_string,
        &args.name,
        &skel_config_path,
    )?;

    if project_config.files.is_none()
        && project_config.dirs.is_none()
        && project_config.build.is_none()
    {
        return Err(Box::from("project dose not have anything to do"));
    }

    let build_first = if args.build_first
        || (project_config.build_first.is_some()
            && project_config.build_first.unwrap())
    {
        true
    } else {
        false
    };

    Ok(Project {
        build_first,
        dirs: project_config.dirs,
        files: project_config.files,
        build: project_config.build,
        templates: project_config.templates,
        skel_config_path,
        name: args.name,
        project_root_path: PathBuf::from(&root_string),
        project_root_string: root_string,
        dont_make_template: args.dont_make_templates,
        dont_run_build: args.dont_run_build,
        show_build_output: args.show_build_output,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::{
        make_fake_conifg_file, make_fake_user_config,
        make_fake_user_config_no_project, TempSetup,
    };

    #[test]
    fn test_config_string_default_or_default() {
        let test_config_paths = default_skel_config_paths();

        let mut config_dir = env::var("HOME")
            .expect("HOME not set")
            .parse::<String>()
            .unwrap();

        // first make the directory
        config_dir.push_str("/.config");
        config_dir.push_str("/skel");

        // then the actual file
        let mut config_file = config_dir.clone();
        config_file.push_str("/config.toml");

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
    fn test_find_project_project_exists() {
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
    fn test_find_project_alias_not_in_config() {
        let config = make_fake_user_config_no_project();

        let project = find_project_file(&config, "test".to_string());

        assert!(project.is_err(), "project some how exists");

        if let Err(err) = project {
            match err.to_string().as_str() {
                "no given alias in user config -- test" => assert!(true),
                _ => assert!(false, "failed for the wrong reason"),
            }
        }
    }

    #[test]
    fn test_find_project_project_dose_not_exists() {
        let config = make_fake_user_config_no_project();

        let project = find_project_file(&config, "cp".to_string());

        assert!(project.is_err(), "project some how exists");

        if let Err(err) = project {
            match err.to_string().as_str() {
                "no project for alias -- cp" => assert!(true),
                _ => assert!(false, "failed for the wrong reason"),
            }
        }
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
            project_path, "/tmp/skel/projects/basic_cpp.toml",
            "failed to template path"
        );
    }

    #[test]
    fn test_get_user_config_from_toml() {
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
    fn test_resolve_project_path_no_user_path() {
        let mut temp = TempSetup::default();
        let root = temp.setup();

        let fake_home = root.to_str().unwrap();
        env::set_var("HOME", fake_home);

        temp.make_fake_user_config()
            .expect("did not make fake config");

        let alias_str = "cpp".to_string();

        let mut fake_config = fake_home.clone().parse::<String>().unwrap();

        fake_config.push_str("/.config");
        fake_config.push_str("/skel");

        let mut fake_config_file = fake_config.clone();

        fake_config_file.push_str("/projects");
        fake_config_file.push_str("/basic_cpp.toml");

        match resolve_project_path(alias_str) {
            Ok((proj_path, proj_dir)) => {
                assert_eq!(
                    proj_path, fake_config_file,
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
