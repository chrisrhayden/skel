use std::{env, error::Error, fs, path::PathBuf, process};

use skel::config::MainConfig;

fn get_main_config() -> Result<PathBuf, Box<dyn Error>> {
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

fn print_items(main_config: MainConfig) {
    for (project, skel) in main_config.skeletons.iter() {
        for a in skel.aliases.iter() {
            print!("{} ", a);
        }

        print!("{} ", project);
    }

    println!();
}

fn run() -> Result<(), Box<dyn Error>> {
    let config_path = get_main_config()?;

    let config_string = fs::read_to_string(config_path)?;

    let main_config: MainConfig = toml::from_str(&config_string)?;

    print_items(main_config);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }

    process::exit(0);
}
