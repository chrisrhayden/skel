use std::{env, error::Error, fs, path::PathBuf, process};

use skel::config::MainConfig;

fn parse_args() -> u8 {
    let mut bit_pattern: u8 = 0;

    for arg in env::args() {
        match arg.as_str() {
            "-a" | "--aliases" => {
                bit_pattern |= 1;
            }
            "-p" | "--projects" => {
                bit_pattern |= 2;
            }
            "-b" | "--both" => {
                bit_pattern |= 3;
            }
            _ => {}
        }
    }

    bit_pattern
}

fn get_main_config() -> Result<PathBuf, Box<dyn Error>> {
    let xdg_config =
        env::var("XDG_CONFIG_HOME").expect("XDG_CONFIG_HOME not set");

    let mut xdg_config_dir = PathBuf::from(&xdg_config);
    xdg_config_dir.push("skel");

    let mut xdg_config_path = xdg_config_dir.clone();
    xdg_config_path.push("config.toml");

    // NOTE: if the file exist then the config dir also exists
    if xdg_config_path.is_file() {
        Ok(xdg_config_path)
    } else {
        Err(Box::from(String::from("main config does not exist")))
    }
}

fn print_aliases(aliases: &[String]) {
    for a in aliases {
        print!("{} ", a);
    }
}

fn print_items(arg_pattern: u8, main_config: &MainConfig) {
    for (project, skel) in main_config.skeletons.iter() {
        if arg_pattern == 0 || arg_pattern == 1 || arg_pattern == 3 {
            print_aliases(&skel.aliases);
        }

        if arg_pattern == 2 || arg_pattern == 3 {
            print!("{} ", project);
        }
    }

    println!();
}

fn run() -> Result<(), Box<dyn Error>> {
    let arg_pattern = parse_args();
    // let config_path = get_main_config()?;

    let config_path =
        PathBuf::from("/home/chris/proj/skel/docs/example.config.toml");

    let config_string = fs::read_to_string(config_path)?;
    let main_config: MainConfig = toml::from_str(&config_string)?;

    print_items(arg_pattern, &main_config);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }

    process::exit(0);
}
