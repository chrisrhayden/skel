use std::{collections::HashMap, env, error::Error};

use serde::Deserialize;

use crate::{
    fs_tools::string_from_file,
    template::{template, TemplateArgs},
    Skeleton, SkeletonConfig,
};

use crate::args::SkelArgs;

#[cfg(unix)]
const PATH_DELIMITER: char = '/';

#[cfg(windows)]
const PATH_DELIMITER: char = '\\';

pub type SkelResult<T> = Result<T, Box<dyn Error>>;

struct ConfigPaths {
    config_file: String,
    config_dir: String,
}

#[derive(Deserialize, Debug)]
pub struct UserConfig {
    pub skeletons: HashMap<String, String>,
    pub alias: HashMap<String, Vec<String>>,
}

// get the root for the users project that the skeleton will be made in to
fn resolve_project_root(name: &str, root_from_cli: Option<String>) -> String {
    // if a root is given at th cli just use that
    let mut r_string = if let Some(from_cli) = root_from_cli {
        from_cli
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
    r_string.push(PATH_DELIMITER);
    r_string.push_str(name);

    r_string
}

fn default_skel_config_paths(cli_config_path: Option<String>) -> ConfigPaths {
    let mut config_dir = env::var("HOME")
        .expect("HOME not set")
        .parse::<String>()
        .expect("cant make path buf from config string");

    // first make the directory
    config_dir.push(PATH_DELIMITER);
    config_dir.push_str(".config");

    config_dir.push(PATH_DELIMITER);
    config_dir.push_str("skel");

    // then the actual file
    let config_file = if let Some(cli_config_path) = cli_config_path {
        cli_config_path
    } else {
        let mut config_file = config_dir.to_owned();

        config_file.push(PATH_DELIMITER);
        config_file.push_str("config.toml");

        config_file
    };

    ConfigPaths {
        config_file,
        config_dir,
    }
}

fn find_skeleton_file(
    user_config: &UserConfig,
    alias_string: String,
) -> SkelResult<String> {
    // if the alias string is an exact project key
    if let Some(path_string) = user_config.skeletons.get(&alias_string) {
        return Ok(path_string.clone());
    }

    // find the project key for the given alias
    // if found clone it to project_string
    for (skeleton, alias) in user_config.alias.iter() {
        if alias.contains(&alias_string) {
            let skeleton_config_path =
                user_config.skeletons.get(skeleton).map(String::from);

            if let Some(config_path) = skeleton_config_path {
                return Ok(config_path);
            } else {
                return Err(Box::from(format!(
                    "no skeleton for given alias in config -- {}",
                    alias_string
                )));
            }
        }
    }

    Err(Box::from(format!(
        "no skeleton for given alias in config -- {}",
        alias_string
    )))
}

fn resolve_config(config_paths: &ConfigPaths) -> SkelResult<UserConfig> {
    let config_str = string_from_file(&config_paths.config_file)?;

    // TODO: let the user know whats wrong in a nice way
    // idk, maybe just say bad config with the file name
    let config = toml::from_str::<UserConfig>(&config_str)
        .unwrap_or_else(|_| panic!("TOML Error -- {}", config_str));

    Ok(config)
}

fn resolve_skeleton_config_path(
    config_paths: &ConfigPaths,
    alias_string: String,
) -> SkelResult<String> {
    let user_config = resolve_config(config_paths)?;

    let raw_config_string = find_skeleton_file(&user_config, alias_string)?;

    let template_args = TemplateArgs {
        project_name: "",
        project_root_path: "",
        skel_config_path: &config_paths.config_dir,
    };

    let skeleton_config_path = template(&template_args, &raw_config_string);

    Ok(skeleton_config_path)
}

// this will iterate over all the given template structs and try and add
// whatever the `include` file contains to the template variables, as of now
// there is no use in keeping the old templates around i guess
pub fn resolve_skeleton_templates(
    skel_config: &mut SkeletonConfig,
    root_path: &str,
    skeleton_name: &str,
    skel_config_path: &str,
) -> Result<(), Box<dyn Error>> {
    if let Some(ref mut temp_files) = skel_config.templates.as_mut() {
        let template_args = TemplateArgs {
            project_root_path: root_path,
            project_name: skeleton_name,
            skel_config_path,
        };

        for template_struct in temp_files.iter_mut() {
            if let Some(include_str) = template_struct.include.as_ref() {
                let template_path = template(&template_args, include_str);

                template_struct.template =
                    Some(string_from_file(template_path)?);

            // if the template exists but does not have a template
            // string or `include` path
            } else if template_struct.include.is_none()
                && template_struct.template.is_none()
            {
                return Err(Box::from(format!(
                    "entry dose not have a template -- name {} -- path {}",
                    skeleton_name, template_struct.path
                )));
            }
        }
    }

    Ok(())
}

// last takes precedent:
//      default > config > cli config
pub fn resolve_defaults(args: SkelArgs) -> SkelResult<Skeleton> {
    // get the root path to make a skeleton in to
    let root_string = resolve_project_root(&args.name, args.different_root);

    let config_paths = default_skel_config_paths(args.cli_config_path);

    let skeleton_str = if let Some(skeleton_file_path) = args.cli_project_file {
        string_from_file(skeleton_file_path)?
    } else {
        let skel_path =
            resolve_skeleton_config_path(&config_paths, args.alias_str)?;

        string_from_file(skel_path)?
    };

    let mut skeleton_config: SkeletonConfig = toml::from_str(&skeleton_str)
        .unwrap_or_else(|_| {
            panic!("Toml Error in skeleton file - {}", skeleton_str)
        });

    resolve_skeleton_templates(
        &mut skeleton_config,
        &root_string,
        &args.name,
        &config_paths.config_dir,
    )?;

    if skeleton_config.files.is_none()
        && skeleton_config.dirs.is_none()
        && skeleton_config.build.is_none()
        && skeleton_config.templates.is_none()
    {
        return Err(Box::from("project dose not have anything to do"));
    }

    // cli overrides everything
    let build_first = args.build_first
        || (
            // make sure there is something before unwrapping
            skeleton_config.build_first.is_some()
            // just unwrap and check the inner bool
            && skeleton_config.build_first.unwrap()
        );

    Ok(Skeleton {
        build_first,
        name: args.name,
        dirs: skeleton_config.dirs,
        files: skeleton_config.files,
        build: skeleton_config.build,
        templates: skeleton_config.templates,
        skel_config_path: config_paths.config_dir,
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
        make_fake_skel_args, make_fake_skeleton_config, make_fake_user_config,
        make_fake_user_config_no_skeleton, TempSetup,
    };

    #[test]
    fn test_default_skel_config_paths_default_path() {
        let args = make_fake_skel_args("test", "test");

        let test_config_paths = default_skel_config_paths(args.cli_config_path);

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

        let project = find_skeleton_file(&config, "cp".to_string());

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_cpp.toml"),
            "failed to find project to make"
        );

        let project = find_skeleton_file(&config, "p".to_string());

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_python.toml"),
            "failed to find project to make"
        );

        let project =
            find_skeleton_file(&config, "basic_javascript".to_string());

        assert!(project.is_ok(), "failed to find project to make");

        assert_eq!(
            project.unwrap(),
            String::from("{{config-dir}}/projects/basic_javascript.toml"),
            "failed to find project to make"
        );
    }

    #[test]
    fn test_find_project_alias_not_in_config() {
        let config = make_fake_user_config_no_skeleton();

        let project = find_skeleton_file(&config, "test".to_string());

        assert!(project.is_err(), "project some how exists");

        if let Err(err) = project {
            println!();
            assert_eq!(
                err.to_string().as_str(),
                "no skeleton for given alias in config -- test",
                "did not get the right error"
            );
        }
    }

    #[test]
    fn test_find_project_project_dose_not_exists() {
        let config = make_fake_user_config_no_skeleton();

        let project = find_skeleton_file(&config, "cp".to_string());

        assert!(project.is_err(), "project some how exists");

        if let Err(err) = project {
            assert_eq!(
                err.to_string().as_str(),
                "no skeleton for given alias in config -- cp",
                "failed for the wrong reason"
            );
        }
    }

    #[test]
    fn test_resolve_config_from_toml() {
        let mut temp = TempSetup::default();
        let root = temp.setup();

        temp.make_fake_user_config().expect("cant make user config");

        let mut temp_config: String = root.to_str().unwrap().to_string();

        temp_config.push_str("/.config");
        temp_config.push_str("/skel");

        let mut temp_config_file = temp_config.to_owned();
        temp_config_file.push_str("/config.toml");

        let config_paths = ConfigPaths {
            config_dir: temp_config,
            config_file: temp_config_file,
        };

        let user_config = match resolve_config(&config_paths) {
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
            user_config.skeletons, projects,
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
    fn test_resolve_skeleton_config_path_no_user_path() {
        let mut temp = TempSetup::default();
        let root = temp.setup();

        let fake_home = root.to_str().unwrap();
        env::set_var("HOME", fake_home);

        temp.make_fake_user_config()
            .expect("did not make fake config");

        let alias_str = "cpp".to_string();

        let mut fake_config = root.to_str().unwrap().parse::<String>().unwrap();

        fake_config.push_str("/.config");
        fake_config.push_str("/skel");

        let mut fake_config_file = fake_config.clone();
        fake_config_file.push_str("/config.toml");

        let mut fake_skeleton_file = fake_config.clone();
        fake_skeleton_file.push_str("/projects");
        fake_skeleton_file.push_str("/basic_cpp.toml");

        let config_paths = ConfigPaths {
            config_dir: fake_config.clone(),
            config_file: fake_config_file.clone(),
        };

        match resolve_skeleton_config_path(&config_paths, alias_str) {
            Ok(proj_path) => {
                assert_eq!(
                    proj_path, fake_skeleton_file,
                    "did not find c++ toml file"
                );
            }
            Err(err) => assert!(false, "Error: {}", err),
        };
    }

    #[test]
    fn test_resolve_skeleton_templates_no_include_file() {
        let mut setup = TempSetup::default();
        let root = setup.setup();

        let root_str = root.to_str().expect("cant get root as str");

        let mut skel_config = make_fake_skeleton_config();

        let mut config_dir = root.clone();

        config_dir.push(".config");
        config_dir.push("skel");

        let resolve_result = resolve_skeleton_templates(
            &mut skel_config,
            root_str,
            "test_project",
            config_dir.to_str().unwrap(),
        );

        assert!(
            resolve_result.is_err(),
            "resolve_skeleton_templates some how got include file"
        );
    }

    #[test]
    fn test_resolve_skeleton_templates_include_file_exits() {
        use std::{fs, io::Write};

        let mut setup = TempSetup::default();
        let root = setup.setup();

        let root_str = root.to_str().expect("cant get root as str");

        let mut skel_config = make_fake_skeleton_config();

        let mut config_dir = root.clone();

        config_dir.push(".config");
        config_dir.push("skel");

        fs::create_dir_all(&config_dir).expect("cant make config dir");

        let mut fake_text = config_dir.clone();

        fake_text.push("test_include.txt");

        let mut text_file =
            fs::File::create(fake_text).expect("cant open include file");

        text_file
            .write_all(b"test include {{name}}")
            .expect("cant write to file");

        let resolve_result = resolve_skeleton_templates(
            &mut skel_config,
            root_str,
            "test_project",
            config_dir.to_str().unwrap(),
        );

        if let Err(err) = resolve_result {
            assert!(
                false,
                "resolve_skeleton_templates returned with and error: {}",
                err
            );
        }

        for temp in skel_config.templates.unwrap() {
            if &temp.path == "src/test_include.txt" {
                let temp_str = temp.template.unwrap();
                assert!(
                    temp_str.starts_with("test include"),
                    "did not get the template file"
                );
            }
        }
    }
}
