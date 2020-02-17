use std::{env, path::PathBuf};

pub fn get_home_dir() -> PathBuf {
    let home_path = env::var("HOME").expect("cant get env var HOME");

    PathBuf::from(home_path)
}
