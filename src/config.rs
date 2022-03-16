use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{parse_args::SkelArgs, templating::instantiate_handlebars};

struct Duplicate {
    key_1: String,
    key_2: String,
    alias: Vec<String>,
}

/// this main config for the program
#[derive(Deserialize, Debug)]
struct MainConfig {
    skeletons: HashMap<String, String>,
    alias: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Default)]
pub struct Template {
    path: PathBuf,
    template: Option<String>,
    include: Option<PathBuf>,
}

/// the run time config
#[derive(Deserialize, Default)]
pub struct SkelConfig {
    pub main_config_path: Option<PathBuf>,
    pub dirs: Option<Vec<PathBuf>>,
    pub files: Option<Vec<PathBuf>>,
    pub templates: Option<Vec<Template>>,
}

fn find_main_config_path(args: &SkelArgs) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(config_path) = &args.main_config_path {
        // Ok(PathBuf::from(config_path))
        Ok(config_path.into())
    } else {
        let xdg_config =
            env::var("XDG_CONFIG_HOME").expect("XDG_CONFIG_HOME not set");

        let mut xdg_config_path: PathBuf = xdg_config.into();
        xdg_config_path.push("skel");
        xdg_config_path.push("config.toml");

        if xdg_config_path.is_file() {
            Ok(xdg_config_path)
        } else {
            Err(Box::from(String::from("main config does not exist")))
        }
    }
}

fn check_config(config: &MainConfig) -> Result<(), Box<dyn Error>> {
    let key_alias: Vec<(&String, &Vec<String>)> = config.alias.iter().collect();

    let mut duplicates = vec![];

    // iterate over all aliases
    for (i, (key_1, value_1)) in key_alias.iter().enumerate() {
        let mut duplicate: Option<Duplicate> = None;

        for (key_2, value_2) in key_alias.iter().skip(i + 1) {
            for s in value_1.iter() {
                if value_2.contains(s) {
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

        // Err(Box::from(String::from("found duplicates")))
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

// TODO: this could probably be better
fn find_skel_config_path(
    skeleton: &str,
    main_config: &MainConfig,
) -> Result<PathBuf, Box<dyn Error>> {
    let skel_config_path =
        if let Some(skel_path) = main_config.skeletons.get(skeleton) {
            Some(skel_path.clone())
        } else {
            let mut skel_path = None;

            for (key, alias) in main_config.alias.iter() {
                if alias.iter().any(|a| a == skeleton) {
                    if let Some(s_path) = main_config.skeletons.get(key) {
                        skel_path = Some(s_path.clone());
                    } else {
                        return Err(Box::from(format!(
                            "no matching project for {}",
                            skeleton
                        )));
                    }

                    break;
                }
            }

            skel_path
        };

    if let Some(skel_path) = skel_config_path {
        let skel_path: PathBuf = skel_path.into();

        if skel_path.is_file() {
            Ok(skel_path)
        } else {
            Err(Box::from(format!(
                "the skeleton path {} does not exists or is not a file",
                skel_path.into_os_string().into_string().unwrap(),
            )))
        }
    } else {
        Err(Box::from(format!(
            "did not find matching skeleton or alias for {}",
            skeleton
        )))
    }
}

fn get_skel_config(
    skel_config_path: &Path,
) -> Result<SkelConfig, Box<dyn Error>> {
    todo!();
}

pub fn resolve_config(args: &SkelArgs) -> Result<SkelConfig, Box<dyn Error>> {
    let main_config_path = find_main_config_path(args)?;

    let main_config = get_main_config(&main_config_path)?;

    let handle = instantiate_handlebars();

    let skel_config_path = find_skel_config_path(&args.skeleton, &main_config)?;

    let skel_config = get_skel_config(&skel_config_path)?;

    Ok(skel_config)
}
