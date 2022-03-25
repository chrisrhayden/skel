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

/// this main config for the program
#[derive(Deserialize, Debug)]
struct MainConfig {
    skeletons: HashMap<String, String>,
    alias: HashMap<String, Vec<String>>,
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
    pub root_string: String,
    pub template_data: Value,
    pub handle: Handlebars<'reg>,
}

fn get_root(args: &SkelArgs) -> Result<String, Box<dyn Error>> {
    if let Some(diff_root) = &args.different_root {
        let diff_root_path = Path::new(diff_root);

        if diff_root_path.is_dir() {
            Ok(diff_root.clone())
        } else {
            Err(Box::from(format!(
                "different root does not exists or is not a dir {}",
                diff_root
            )))
        }
    } else {
        let root_path = env::current_dir().expect("could not get current dir");

        let root_string = root_path
            .to_str()
            .expect("could not parse root string")
            .to_string();

        Ok(root_string)
    }
}

fn find_main_config_path(args: &SkelArgs) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(config_path) = &args.main_config_path {
        Ok(config_path.into())
    } else {
        let xdg_config =
            env::var("XDG_CONFIG_HOME").expect("XDG_CONFIG_HOME not set");

        let mut xdg_config_path = PathBuf::from(&xdg_config);
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
            // TODO: check for keys here maybe
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

fn skeleton_path_from_config(
    skeleton: &str,
    main_config: &MainConfig,
) -> Result<String, Box<dyn Error>> {
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
        Ok(skel_path)
    } else {
        Err(Box::from(format!(
            "did not find matching skeleton or alias for {}",
            skeleton
        )))
    }
}

fn find_skel_config_path(
    args: &SkelArgs,
    main_config: &MainConfig,
    handle: &Handlebars,
    template_data: &Value,
) -> Result<PathBuf, Box<dyn Error>> {
    let skel_string = if let Some(skeleton) = args.skeleton.as_ref() {
        skeleton_path_from_config(skeleton, main_config)?
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

fn get_skel_config(
    skel_config_path: &Path,
) -> Result<SkelConfig, Box<dyn Error>> {
    let skel_config_buf = fs::read(skel_config_path)?;

    toml::from_slice(&skel_config_buf).map_err(Box::from)
}

pub fn resolve_config(args: &SkelArgs) -> Result<RunConfig, Box<dyn Error>> {
    let root_string = get_root(args)?;

    let main_config_path = find_main_config_path(args)?;

    let main_config = get_main_config(&main_config_path)?;

    let main_config_parent = match main_config_path.parent() {
        None => return Err(Box::from(String::from("cant make files in root"))),
        Some(value) => match value.as_os_str().to_str() {
            None => {
                return Err(Box::from(String::from(
                    "cant get a string from os_str",
                )))
            }
            Some(value) => value.to_string(),
        },
    };

    let template_data = json!(TempleData {
        root: root_string.clone(),
        name: args.name.clone(),
        config_dir: main_config_parent,
    });

    let handle = instantiate_handlebars();

    let skel_config_path =
        find_skel_config_path(args, &main_config, &handle, &template_data)?;

    let skel_conf = get_skel_config(&skel_config_path)?;

    let run_conf = RunConfig {
        skel_conf,
        root_string,
        template_data,
        handle,
    };

    Ok(run_conf)
}
