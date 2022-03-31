use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use handlebars::Handlebars;

use serde::Deserialize;

use serde_json::{json, Value};

use crate::{
    parse_args::SkelArgs,
    templating::{instantiate_handlebars, TempleData},
};

struct Duplicate {
    key_1: String,
    key_2: String,
    alias: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Skeleton {
    path: String,
    aliases: Vec<String>,
}

/// this main config for the program
#[derive(Deserialize, Debug)]
struct MainConfig {
    skeletons: HashMap<String, Skeleton>,
}

#[derive(Deserialize, Default)]
pub struct SkelTemplate {
    pub path: String,
    pub template: Option<String>,
    pub include: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct SkelConfig {
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub templates: Option<Vec<SkelTemplate>>,
}

#[derive(Default)]
pub struct RunConfig<'reg> {
    pub skel_conf: SkelConfig,
    pub root_path: PathBuf,
    pub template_data: Value,
    pub handle: Handlebars<'reg>,
}

fn find_main_config_path(
    args: &SkelArgs,
) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    if let Some(config_string) = &args.main_config_path {
        let config_path = PathBuf::from(config_string);

        if !config_path.is_file() {
            return Err(Box::from(format!(
                "given config path does not exist or is not a file {}",
                config_string
            )));
        }

        // NOTE: this is probably fine as we check if the path is explicitly a
        // file so this should at least return root
        let config_dir = config_path.parent().unwrap().to_owned();

        Ok((config_dir, config_path))
    } else {
        let xdg_config =
            env::var("XDG_CONFIG_HOME").expect("XDG_CONFIG_HOME not set");

        let mut xdg_config_dir = PathBuf::from(&xdg_config);
        xdg_config_dir.push("skel");

        let mut xdg_config_path = xdg_config_dir.clone();
        xdg_config_path.push("config.toml");

        // NOTE: if the file exist then the config dir also exists
        if xdg_config_path.is_file() {
            Ok((xdg_config_dir, xdg_config_path))
        } else {
            Err(Box::from(String::from("main config does not exist")))
        }
    }
}

fn check_config(config: &MainConfig) -> Result<(), Box<dyn Error>> {
    let key_alias: Vec<(&String, &Skeleton)> =
        config.skeletons.iter().collect();

    let mut duplicates = vec![];

    for (i, (key_1, value_1)) in key_alias.iter().enumerate() {
        let mut duplicate: Option<Duplicate> = None;

        for (key_2, value_2) in key_alias.iter().skip(i + 1) {
            for s in value_1.aliases.iter() {
                if value_2.aliases.contains(s) {
                    if let Some(duplicate) = &mut duplicate {
                        duplicate.alias.push(s.clone());
                    } else {
                        let dup = Duplicate {
                            key_1: key_1.to_string(),
                            key_2: key_2.to_string(),
                            alias: vec![s.clone()],
                        };

                        duplicate = Some(dup);
                    }
                }
            }
        }

        if let Some(duplicate) = duplicate {
            duplicates.push(duplicate);
        }
    }

    if !duplicates.is_empty() {
        let mut dup_str = String::from("duplicate keys found\n");
        let duplicates_len = duplicates.len() - 1;

        for (i, dup) in duplicates.into_iter().enumerate() {
            let new_dup = format!(
                "keys [\x1b[31m{}\x1b[0m, \x1b[31m{}\x1b[0m]\n    alias: [\x1b[31m{}\x1b[0m]",
                dup.key_1,
                dup.key_2,
                dup.alias.join(", ")
            );

            dup_str.push_str(&new_dup);
            if i != duplicates_len {
                dup_str.push('\n');
            }
        }

        Err(Box::from(dup_str))
    } else {
        Ok(())
    }
}

fn get_main_config(
    main_config_path: &Path,
) -> Result<MainConfig, Box<dyn Error>> {
    let config_string = fs::read(main_config_path)
        .expect("Config does not exist or is not a file");

    let config: MainConfig = toml::from_slice(&config_string)
        .expect("config not formatted correctly");

    check_config(&config)?;

    Ok(config)
}

fn skeleton_path_from_config(
    target: &str,
    main_config: &MainConfig,
) -> Result<String, Box<dyn Error>> {
    let skel_config_path =
        if let Some(skeleton) = main_config.skeletons.get(target) {
            Some(skeleton.path.clone())
        } else {
            let mut skel_path = None;

            for (_, skeleton) in main_config.skeletons.iter() {
                if skeleton.aliases.iter().any(|a| a == target) {
                    skel_path = Some(skeleton.path.clone());

                    break;
                }
            }

            skel_path
        };

    if let Some(skel_path) = skel_config_path {
        Ok(skel_path)
    } else {
        Err(Box::from(format!(
            "did not find matching skeleton or alias for {}",
            target
        )))
    }
}

fn find_skeleton_config_path(
    args: &SkelArgs,
    main_config: &MainConfig,
    handle: &Handlebars,
    template_data: &Value,
) -> Result<PathBuf, Box<dyn Error>> {
    // a skeleton target/project/aliases
    let skel_string = if let Some(target) = args.skeleton.as_ref() {
        skeleton_path_from_config(target, main_config)?
    // a file given on the cli
    } else if let Some(skeleton_file) = args.skeleton_file.as_ref() {
        skeleton_file.clone()
    } else {
        return Err(Box::from(String::from(
            "did not get  skeleton to make some how",
        )));
    };

    let skel_templated = handle.render_template(&skel_string, template_data)?;

    let skel_path: PathBuf = PathBuf::from(&skel_templated);

    if skel_path.is_file() {
        Ok(skel_path)
    } else {
        Err(Box::from(format!(
            "skeleton file does not exist or is not a file {}",
            skel_templated
        )))
    }
}

fn get_skel_config<P: AsRef<Path>>(
    skel_config_path: P,
    handle: &Handlebars,
    template_data: &Value,
) -> Result<SkelConfig, Box<dyn Error>> {
    let skel_config_buf = fs::read_to_string(skel_config_path)?;

    let templated_config_string = handle
        .render_template(&skel_config_buf, template_data)
        .expect("was not able to template skeleton");

    toml::from_str(&templated_config_string).map_err(Box::from)
}

pub fn resolve_config<'reg>(
    args: &SkelArgs,
    root_string: PathBuf,
) -> Result<RunConfig<'reg>, Box<dyn Error>> {
    let (main_config_dir, main_config_path) = find_main_config_path(args)?;

    let main_config = get_main_config(&main_config_path)?;

    let template_data = json!(TempleData {
        root: root_string.as_os_str().to_str().unwrap().to_string(),
        name: args.name.to_string(),
        config_dir: main_config_dir.as_os_str().to_str().unwrap().to_string(),
    });

    let handle = instantiate_handlebars();

    let skel_config_path =
        find_skeleton_config_path(args, &main_config, &handle, &template_data)?;

    let skel_conf =
        get_skel_config(&skel_config_path, &handle, &template_data)?;

    let run_conf = RunConfig {
        skel_conf,
        root_path: root_string,
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

        let test_key_1 = test_utils::TEST_PROJECT_NAME.into();
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

    fn fake_main_config() -> MainConfig {
        let mut main_config = MainConfig {
            skeletons: HashMap::new(),
        };

        let test_key_1 = test_utils::TEST_PROJECT_NAME.into();
        let test_skeleton_1 = Skeleton {
            path: test_utils::TEST_PROJECT_PATH.into(),
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

        match find_main_config_path(&test_args) {
            Ok((parent, config_path)) => {
                assert_eq!(config_path, test_config_path);

                assert_eq!(parent, test_config_path.parent().unwrap());
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

        test_args.main_config_path =
            Some(test_config_path.as_os_str().to_str().unwrap().to_string());
        match find_main_config_path(&test_args) {
            Ok((parent, config_path)) => {
                assert_eq!(config_path, test_config_path);

                assert_eq!(parent, test_config_path.parent().unwrap());
            }
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_find_main_config_no_config() {
        let test_args = test_utils::test_args();

        assert!(
            find_main_config_path(&test_args).is_err(),
            "some how found a config?"
        );
    }

    #[test]
    fn test_check_config() {
        let main_config = fake_main_config();

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

            let test_path =
                "{{config_dir}}/projects/test_project.toml".to_string();

            let test_aliases = vec![
                "t".to_string(),
                "T".to_string(),
                "test_project".to_string(),
            ];

            if let Ok(main_config) = get_main_config(main_config_path) {
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
        let main_config = fake_main_config();

        let skel_path = skeleton_path_from_config(
            test_utils::TEST_PROJECT_NAME,
            &main_config,
        )
        .expect("did not find config");

        assert_eq!(
            skel_path,
            test_utils::TEST_PROJECT_PATH,
            "did not get the correct skeleton path"
        );
    }

    #[test]
    fn test_skeleton_path_from_config_alias_exists() {
        let main_config = fake_main_config();

        let target: String = "t".into();

        let skel_path = skeleton_path_from_config(&target, &main_config)
            .expect("did not find config");

        assert_eq!(
            skel_path,
            test_utils::TEST_PROJECT_PATH,
            "did not get the correct skeleton path"
        );
    }

    #[test]
    fn test_skeleton_path_from_config_does_not_exist() {
        let main_config = fake_main_config();

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

        let main_config = fake_main_config();

        let handle = instantiate_handlebars();

        let template_data = json!(TempleData {
            root: "".into(),
            name: args.name.to_string(),
            config_dir: test_data.temp_path_string.clone(),
        });

        let hand_made_skel_path = handle
            .render_template(test_utils::TEST_PROJECT_PATH, &template_data)
            .unwrap();

        let hand_made_skel_path = PathBuf::from(hand_made_skel_path);

        match find_skeleton_config_path(
            &args,
            &main_config,
            &handle,
            &template_data,
        ) {
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

        let main_config = fake_main_config();

        let handle = instantiate_handlebars();

        let template_data = json!(TempleData {
            root: "".into(),
            name: args.name.to_string(),
            config_dir: test_data.temp_path_string.clone(),
        });

        let hand_made_skel_path = PathBuf::from(skel_file);

        fs::File::create(&hand_made_skel_path).unwrap();

        match find_skeleton_config_path(
            &args,
            &main_config,
            &handle,
            &template_data,
        ) {
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

        let template_data = json!(TempleData {
            root: "".into(),
            name: "test_project".into(),
            config_dir: test_data.temp_path_string.clone(),
        });

        let handle = instantiate_handlebars();

        let skel_config_path = handle
            .render_template(test_utils::TEST_PROJECT_PATH, &template_data)
            .unwrap();

        if let Err(err) =
            get_skel_config(&skel_config_path, &handle, &template_data)
        {
            panic!("{}", err);
        }
    }

    #[test]
    fn test_get_skel_config_does_not_exists() {
        let template_data = json!(TempleData {
            root: "".into(),
            name: "test_project".into(),
            config_dir: "test_config_dir".into(),
        });

        let handle = instantiate_handlebars();

        if get_skel_config("/tmp/does_not_exists.toml", &handle, &template_data)
            .is_ok()
        {
            panic!("some how config exists");
        }
    }
}
