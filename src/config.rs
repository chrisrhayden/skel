use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use handlebars::Handlebars;

use serde::Deserialize;

use crate::{parse_args::SkelArgs, templating::instantiate_handlebars};

/// the path and alias to find a skeleton file
#[derive(Deserialize, Debug)]
pub struct Skeleton {
    pub path: String,
    pub aliases: Vec<String>,
}

/// this is them main config for the program
#[derive(Deserialize, Debug)]
pub struct MainConfig {
    pub skeletons: HashMap<String, Skeleton>,
}

/// a file template
#[derive(Deserialize, Default)]
pub struct SkelTemplate {
    pub path: String,
    pub template: Option<String>,
    pub include: Option<String>,
}

/// a skeleton
#[derive(Deserialize, Default)]
pub struct SkelConfig {
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub templates: Option<Vec<SkelTemplate>>,
    pub build: Option<String>,
    pub build_first: Option<bool>,
}

/// the needed data to make the project
#[derive(Default)]
pub struct RunConfig<'reg> {
    pub skel_conf: SkelConfig,
    pub root_path: PathBuf,
    pub template_data: HashMap<String, String>,
    pub handle: Handlebars<'reg>,
}

// a struct to hold duplicate values in a main config
struct Duplicate<'a> {
    key_1: &'a str,
    key_2: &'a str,
    alias: Vec<&'a str>,
}

fn make_duplicate_err_msg(duplicates: &[Duplicate]) -> String {
    let mut dup_str = String::from("duplicate keys or aliases found\n");
    let duplicates_len = duplicates.len() - 1;

    for (i, dup) in duplicates.iter().enumerate() {
        let new_dup = format!(
                "keys [\x1b[31m{}\x1b[0m, \x1b[31m{}\x1b[0m]\n    alias: [\x1b[31m{}\x1b[0m]",
                dup.key_1,
                dup.key_2,
                dup.alias.join(", ")
            );

        dup_str.push_str(&new_dup);

        // don't add a new line for the last item
        if i != duplicates_len {
            dup_str.push('\n');
        }
    }

    dup_str
}

// check for duplicates in the main config
//
// NOTE: it would be more efficient to check for the target skeleton in this
// function but whatever
fn check_config(config: &MainConfig) -> Result<(), Box<dyn Error>> {
    // collect all the skeletons in to a vec so we can iterate over them in
    // order, this is kinda bad but it is also the easiest
    let key_alias: Vec<(&String, &Skeleton)> =
        config.skeletons.iter().collect();

    let mut duplicates = vec![];

    // iterate over all the skeletons in the main config then iterate over the
    // following ones checking if there any duplicates
    //
    // we can skip past the previous skeletons in the inner loop as the
    // previous skeletons have already been checked for duplicates
    for (i, (key_1, skeleton_1)) in key_alias.iter().enumerate() {
        for (key_2, skeleton_2) in key_alias.iter().skip(i + 1) {
            let mut duplicate: Option<Duplicate> = None;

            // NOTE: i guess multiple keys will be a parsing error
            if key_1 == key_2 {
                let dup = Duplicate {
                    key_1,
                    key_2,
                    alias: vec![],
                };

                duplicate = Some(dup);
            }

            // iterate over the outer value's aliases and check if any are in
            // the inside value
            for s in skeleton_1.aliases.iter() {
                if skeleton_2.aliases.contains(s) {
                    if let Some(ref mut duplicate) = duplicate {
                        duplicate.alias.push(s);
                    } else {
                        let dup = Duplicate {
                            key_1,
                            key_2,
                            alias: vec![s],
                        };

                        duplicate = Some(dup);
                    }
                }
            }

            if let Some(duplicate) = duplicate {
                duplicates.push(duplicate);
            }
        }
    }

    if !duplicates.is_empty() {
        let dup_err_msg = make_duplicate_err_msg(&duplicates);

        Err(Box::from(dup_err_msg))
    } else {
        Ok(())
    }
}

// get that path to the main config
fn get_main_config_path(args: &SkelArgs) -> Result<PathBuf, Box<dyn Error>> {
    // first check if an alternate config path is given
    let main_config_path = if let Some(ref config_string) = args.alt_config_path
    {
        PathBuf::from(config_string)

    // else try and use xdg_config_home
    } else {
        let mut xdg_config = match env::var("XDG_CONFIG_HOME") {
            Ok(path) => PathBuf::from(path),
            _ => {
                // this is fine on linux and probably mac
                // but will fail on windows
                let home = env::var("HOME")?;

                let mut home_path = PathBuf::from(home);

                home_path.push(".config");

                home_path
            }
        };

        xdg_config.push("skel");

        xdg_config.push("config.toml");

        xdg_config
    };

    if main_config_path.is_file() {
        Ok(main_config_path)
    } else {
        Err(Box::from(format!(
            "given config path does not exist or is not a file {}",
            main_config_path.display()
        )))
    }
}

// get the main config file from a given path and return it
fn get_main_config(
    main_config_path: &Path,
    handle: &Handlebars,
    template_data: &HashMap<String, String>,
) -> Result<MainConfig, Box<dyn Error>> {
    let config_string = fs::read_to_string(main_config_path)?;

    let templated_config_string =
        handle.render_template(&config_string, &template_data)?;

    let config: MainConfig = toml::from_str(&templated_config_string)?;

    check_config(&config)?;

    Ok(config)
}

// retrieve the skeleton path from the main config
fn skeleton_path_from_config(
    target: &str,
    main_config: &MainConfig,
) -> Result<String, Box<dyn Error>> {
    let skel_config_path =
        if let Some(skeleton) = main_config.skeletons.get(target) {
            Some(skeleton.path.clone())
        } else {
            let mut skel_path = None;

            // check all the aliases to see if one matches
            for (_, skeleton) in main_config.skeletons.iter() {
                if skeleton.aliases.iter().any(|s| s == target) {
                    skel_path = Some(skeleton.path.clone());

                    break;
                }
            }

            skel_path
        };

    skel_config_path.ok_or_else(|| {
        Box::from(format!(
            "did not find matching skeleton or alias for {}",
            target
        ))
    })
}

// find the skeleton config path and check if the file exists
fn get_skeleton_config_path(
    args: &SkelArgs,
    main_config: &MainConfig,
) -> Result<PathBuf, Box<dyn Error>> {
    // a file given on the cli
    let skel_path = if let Some(skeleton_file) = args.skeleton_file.as_ref() {
        let skel_path = PathBuf::from(skeleton_file);

        skel_path
    // a skeleton project or alias
    } else if let Some(target) = args.skeleton.as_ref() {
        let skel_path = skeleton_path_from_config(target, main_config)?;

        PathBuf::from(skel_path)
    } else {
        return Err(Box::from(String::from(
            "did not get skeleton to make some how",
        )));
    };

    if skel_path.is_file() {
        Ok(skel_path)
    } else {
        Err(Box::from(format!(
            "skeleton file does not exist or is not a file {}",
            skel_path.to_string_lossy().to_string()
        )))
    }
}

fn make_skel_config<P: AsRef<Path>>(
    skel_config_path: P,
    handle: &Handlebars,
    template_data: &HashMap<String, String>,
) -> Result<SkelConfig, Box<dyn Error>> {
    let skel_config_buf = fs::read_to_string(skel_config_path)?;

    let templated_config_string = handle
        .render_template(&skel_config_buf, template_data)
        .expect("was not able to template skeleton");

    toml::from_str(&templated_config_string).map_err(|e| {
        Box::from(format!("skeleton config not formatted correctly {}", e))
    })
}

/// resolve config
///
/// # Arguments
///
/// * `args` - cli args
/// * `root_path` - the path to make skel in to
/// * `name` - the name of the new project
pub fn resolve_config<'reg>(
    args: &SkelArgs,
    root_path: PathBuf,
    name: String,
) -> Result<RunConfig<'reg>, Box<dyn Error>> {
    let main_config_path = get_main_config_path(args)?;

    let main_config_dir = main_config_path
        .parent()
        .ok_or("could not get the parent dir for the main config")?;

    let handle = instantiate_handlebars();

    let mut template_data: HashMap<String, String> = HashMap::new();

    template_data.insert("name".to_string(), name);

    template_data
        .insert("root".to_string(), root_path.to_string_lossy().to_string());

    template_data.insert(
        "config-dir".to_string(),
        main_config_dir.to_string_lossy().to_string(),
    );

    let skel_config_path = if let Some(ref skeleton_file) = args.skeleton_file {
        PathBuf::from(skeleton_file)
    } else {
        let main_config =
            get_main_config(&main_config_path, &handle, &template_data)?;

        get_skeleton_config_path(args, &main_config)?
    };

    template_data.insert(
        "skel-dir".to_string(),
        skel_config_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    let skel_conf =
        make_skel_config(&skel_config_path, &handle, &template_data)?;

    let run_conf = RunConfig {
        skel_conf,
        root_path,
        template_data,
        handle,
    };

    Ok(run_conf)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils;

    fn fake_main_config_duplicate() -> MainConfig {
        let mut main_config = MainConfig {
            skeletons: HashMap::new(),
        };

        let test_key_1 = test_utils::TEST_PROJECT_KEY.into();
        let test_skeleton_1 = Skeleton {
            path: test_utils::TEST_PROJECT_PATH.into(),
            aliases: test_utils::TEST_PROJECT_ALIASES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        main_config.skeletons.insert(test_key_1, test_skeleton_1);

        let test_key_2 = "another_test_key".into();
        let test_skeleton_2 = Skeleton {
            path: test_utils::TEST_PROJECT_PATH.into(),
            aliases: test_utils::TEST_PROJECT_ALIASES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        main_config.skeletons.insert(test_key_2, test_skeleton_2);

        main_config
    }

    fn fake_main_config(test_data: &test_utils::TestData) -> MainConfig {
        let mut main_config = MainConfig {
            skeletons: HashMap::new(),
        };

        let test_key_1 = test_utils::TEST_PROJECT_KEY.into();

        let test_skeleton_1 = Skeleton {
            path: test_utils::TEST_PROJECT_PATH
                .replace("{{config-dir}}", &test_data.temp_path_string),
            aliases: test_utils::TEST_PROJECT_ALIASES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        main_config.skeletons.insert(test_key_1, test_skeleton_1);

        let test_key_2 = "test_key_2".into();
        let test_skeleton_2 = Skeleton {
            path: "test_project_2.toml".into(),
            aliases: vec!["a".into(), "A".into()],
        };

        main_config.skeletons.insert(test_key_2, test_skeleton_2);

        main_config
    }

    #[test]
    fn test_find_main_config_default() {
        let mut test_data = test_utils::TestData::default();
        test_data.make_configs();

        let test_args = test_utils::test_args();

        let mut test_config_path = test_data.temp_path;

        test_config_path.push("skel/config.toml");

        match get_main_config_path(&test_args) {
            Ok(config_path) => {
                assert_eq!(config_path, test_config_path);
            }
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_find_main_config_args() {
        let mut test_data = test_utils::TestData::default();

        test_data.make_configs();

        let mut test_args = test_utils::test_args();
        let mut test_config_path = test_data.temp_path;

        test_config_path.push("test_config.toml");

        fs::write(&test_config_path, test_utils::TEST_CONFIG)
            .expect("could not make test config");

        test_args.alt_config_path =
            Some(test_config_path.as_os_str().to_str().unwrap().to_string());

        match get_main_config_path(&test_args) {
            Ok(config_path) => {
                assert_eq!(config_path, test_config_path);
            }
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_find_main_config_no_config() {
        let test_args = test_utils::test_args();

        assert!(
            get_main_config_path(&test_args).is_err(),
            "some how found a config?"
        );
    }

    #[test]
    fn test_check_config() {
        let test_data = test_utils::TestData::default();

        let main_config = fake_main_config(&test_data);

        assert!(
            check_config(&main_config).is_ok(),
            "did not find duplicate aliases"
        );
    }

    #[test]
    fn test_check_config_duplicates() {
        let main_config = fake_main_config_duplicate();

        assert!(
            check_config(&main_config).is_err(),
            "did not find duplicate aliases"
        );
    }

    #[test]
    fn test_get_main_config() {
        let mut test_data = test_utils::TestData::default();

        test_data.make_configs();

        if let Some(main_config_path) = test_data.test_main_config_path.as_ref()
        {
            // let test_project_key: String = "test_project".into();

            let test_path = format!(
                "{}/projects/test_project.toml",
                test_data.temp_path_string
            );

            let test_aliases = vec![
                "t".to_string(),
                "T".to_string(),
                "test_project".to_string(),
            ];

            let mut template_data = HashMap::new();
            template_data.insert("root".to_string(), "".to_string());
            template_data
                .insert("name".to_string(), "test_project".to_string());
            template_data.insert(
                "config-dir".to_string(),
                test_data.temp_path_string.clone(),
            );

            let handle = instantiate_handlebars();

            if let Ok(main_config) =
                get_main_config(main_config_path, &handle, &template_data)
            {
                let test_project =
                    main_config.skeletons.get("test_project").unwrap();

                assert_eq!(
                    test_project.path, test_path,
                    "did not parse config file correctly"
                );

                assert_eq!(
                    test_project.aliases, test_aliases,
                    "did not parse config correctly"
                );
            } else {
                panic!("did not deserialize main config from file");
            }
        } else {
            panic!("some how did not make main config file");
        }
    }

    #[test]
    fn test_skeleton_path_from_config_project_exists() {
        let test_data = test_utils::TestData::default();
        let main_config = fake_main_config(&test_data);

        let hand_made_project_path = test_utils::TEST_PROJECT_PATH
            .replace("{{config-dir}}", &test_data.temp_path_string);

        let skel_path = skeleton_path_from_config(
            test_utils::TEST_PROJECT_KEY,
            &main_config,
        )
        .expect("did not find config");

        assert_eq!(
            skel_path, hand_made_project_path,
            "did not get the correct skeleton path"
        );
    }

    #[test]
    fn test_skeleton_path_from_config_alias_exists() {
        let test_data = test_utils::TestData::default();
        let main_config = fake_main_config(&test_data);

        let target: String = "t".into();

        let hand_made_project_path = test_utils::TEST_PROJECT_PATH
            .replace("{{config-dir}}", &test_data.temp_path_string);

        let skel_path = skeleton_path_from_config(&target, &main_config)
            .expect("did not find config");

        assert_eq!(
            skel_path, hand_made_project_path,
            "did not get the correct skeleton path"
        );
    }

    #[test]
    fn test_skeleton_path_from_config_does_not_exist() {
        let test_data = test_utils::TestData::default();
        let main_config = fake_main_config(&test_data);

        let target: String = "does_not_exist".into();

        assert!(
            skeleton_path_from_config(&target, &main_config).is_err(),
            "project  some how exists"
        );
    }

    #[test]
    fn test_find_skeleton_config_path_from_aliases() {
        let mut test_data = test_utils::TestData::default();

        test_data.make_configs();

        let args = test_utils::test_args();

        let main_config = fake_main_config(&test_data);

        let mut hand_made_skel_path = test_data.temp_path.clone();

        hand_made_skel_path.push("projects");
        hand_made_skel_path.push(test_utils::TEST_SKEL_NAME);

        match get_skeleton_config_path(&args, &main_config) {
            Ok(config_dir) => assert_eq!(
                config_dir, hand_made_skel_path,
                "did not make skeleton path"
            ),
            Err(err) => panic!("did not find skeleton path {}", err),
        }
    }

    #[test]
    fn test_find_skeleton_config_path_from_args() {
        let mut test_data = test_utils::TestData::default();

        test_data.make_configs();

        let mut args = test_utils::test_args();

        args.skeleton = None;

        let mut skel_file = test_data.temp_path_string.clone();

        // TODO: fix if windows support is added
        skel_file.push_str("/test_skeleton_2.toml");

        args.skeleton_file = Some(skel_file.clone());

        let main_config = fake_main_config(&test_data);

        let hand_made_skel_path = PathBuf::from(skel_file);

        fs::File::create(&hand_made_skel_path).unwrap();

        match get_skeleton_config_path(&args, &main_config) {
            Ok(config_dir) => {
                assert_eq!(
                    config_dir, hand_made_skel_path,
                    "did not make skeleton path"
                )
            }
            Err(err) => panic!("did not find skeleton path {}", err),
        }
    }

    #[test]
    fn test_get_skel_config_exists() {
        let mut test_data = test_utils::TestData::default();

        test_data.make_configs();

        let mut template_data = HashMap::new();
        template_data.insert("root".to_string(), "".to_string());
        template_data.insert("name".to_string(), "test_project".to_string());
        template_data.insert(
            "config-dir".to_string(),
            test_data.temp_path_string.clone(),
        );

        let handle = instantiate_handlebars();

        let skel_config_path = handle
            .render_template(test_utils::TEST_PROJECT_PATH, &template_data)
            .unwrap();

        if let Err(err) =
            make_skel_config(&skel_config_path, &handle, &template_data)
        {
            panic!("{}", err);
        }
    }

    #[test]
    fn test_get_skel_config_does_not_exists() {
        let mut template_data = HashMap::new();
        template_data.insert("root".to_string(), "".to_string());
        template_data.insert("name".to_string(), "test_project".to_string());
        template_data
            .insert("config-dir".to_string(), "test_config_dir".to_string());

        let handle = instantiate_handlebars();

        if make_skel_config(
            "/tmp/does_not_exists.toml",
            &handle,
            &template_data,
        )
        .is_ok()
        {
            panic!("some how config exists");
        }
    }
}
