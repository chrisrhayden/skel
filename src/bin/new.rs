use std::env;

use new_rs::make_project;
use new_rs::{collect_config, Project};

fn main() {
    let mut proj_src = env::current_dir().expect("cant get current dir");

    proj_src.push("fake_js_proj.toml");

    let config = match collect_config(&proj_src) {
        Err(err) => {
            eprintln!("Config Error: {}", err);
            return;
        }
        Ok(val) => val,
    };

    let name = Some(String::from("test_project"));

    let project = Project::new(None, name, config);

    if let Err(err) = make_project(&project) {
        eprintln!("{}", err);
    }
}
