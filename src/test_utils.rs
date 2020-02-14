#![allow(dead_code)]
use std::path::PathBuf;

use tempfile::{tempdir, TempDir};

use crate::project::{Project, ProjectConfig, Template};

#[derive(Default)]
pub struct TempSetup {
    path: PathBuf,
    temp: Option<TempDir>,
}

impl TempSetup {
    pub fn setup(&mut self) -> PathBuf {
        self.temp = Some(tempdir().unwrap());
        self.path = self.temp.as_ref().unwrap().path().to_owned();

        self.path.clone()
    }

    pub fn pathbuf(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Drop for TempSetup {
    fn drop(&mut self) {
        if let Some(temp) = self.temp.take() {
            temp.close().expect("cant close file");
        }
    }
}

pub fn make_fake_config() -> ProjectConfig {
    let name = Some(String::from("test_project"));

    let dirs: Vec<String> = vec![
        String::from("src"),
        String::from("tests"),
        String::from("tests/more_tests"),
    ];

    let files: Vec<String> = vec![
        String::from("src/main.rs"),
        String::from("tests/test_main.rs"),
    ];

    // let build: Option<String> = Some("echo 'test shell'".to_string());
    let build = Some(String::from(
        r#"if [[ -f test_project ]]; then
    echo "test_project exists"
fi"#,
    ));

    let template_one = Template {
        name: String::from("src/main.rs"),
        template: r#"fn main() {
    println!("hello world");
}
"#
        .to_string(),
    };

    let template_two = Template {
        name: String::from("tests/test_main.rs"),
        template: String::from("no tests yet"),
    };

    ProjectConfig {
        name,
        dirs,
        files,
        build,
        templates: vec![template_one, template_two],
    }
}

pub fn make_fake_project(root: Option<PathBuf>) -> Project {
    let root: String = if let Some(root) = root {
        String::from(
            root.as_os_str().to_str().expect("cant get temp path a str"),
        )
    } else {
        String::from("/tmp/test_root")
    };

    let conf = make_fake_config();

    let name = Some(String::from("test_project"));

    Project::new(Some(root), name, conf)
}
