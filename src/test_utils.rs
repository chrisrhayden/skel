#![allow(dead_code)]
use std::{error::Error, fs, path::PathBuf};

use tempfile::{tempdir, TempDir};

use toml;

use crate::project::{Project, ProjectConfig};

#[derive(Default)]
pub struct TempSetup {
    root: PathBuf,
    temp: Option<TempDir>,
    project: Option<Project>,
}

impl TempSetup {
    pub fn setup(&mut self) -> PathBuf {
        self.temp = Some(tempdir().unwrap());
        self.root = self.temp.as_ref().unwrap().path().to_owned();

        self.project = Some(make_fake_project(Some(self.root.clone())));

        self.root.clone()
    }

    pub fn root_buf(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn make_fake_project_dirs(
        &self,
        proj: Option<&Project>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.root.exists() {
            panic!("must be run after setup");
        }

        let project = if let Some(proj) = proj {
            proj
        } else {
            self.project.as_ref().unwrap()
        };

        for dir in project.dir_iter() {
            fs::create_dir_all(dir)?;
        }

        Ok(())
    }

    pub fn make_fake_project_files(
        &self,
        proj: Option<&Project>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.root.exists() {
            panic!("must be run after setup");
        }

        let project = if let Some(proj) = proj {
            proj
        } else {
            self.project.as_ref().unwrap()
        };

        for file in project.file_iter() {
            fs::File::create(file)?;
        }

        Ok(())
    }

    // im not sure what would be a better way
    pub fn make_fake_project_tree(&self) -> Result<(), Box<dyn Error>> {
        if !self.root.exists() {
            panic!("must be run after setup");
        }

        let project = self.project.as_ref();

        if let Err(err) = self.make_fake_project_dirs(project) {
            eprintln!("{}", err);
            panic!("can t make dirs {}", err);
        }

        if let Err(err) = self.make_fake_project_files(project) {
            eprintln!("{}", err);
            panic!("can t make files {}", err);
        }

        // TODO: add templating

        Ok(())
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
    let fake_toml = make_fake_toml();

    toml::from_str::<ProjectConfig>(&fake_toml)
        .expect("cant make config from fake toml")
}

pub fn make_fake_project(root: Option<PathBuf>) -> Project {
    let mut root: String = if let Some(root) = root {
        String::from(
            root.as_os_str().to_str().expect("cant get temp path a str"),
        )
    } else {
        String::from("/tmp/test_root")
    };

    let conf = make_fake_config();

    let name = String::from("test_project");

    root.push('/');
    root.push_str(&name);

    Project::new(root, name, conf)
}

pub fn make_fake_toml() -> String {
    r#"dirs = [
    "src",
    "tests",
    "tests/more_tests"
]
files = [
    "src/main.rs",
    "tests/test_main.rs"
]

build = """
if [[ -d test_project ]]; then
    echo "running in $PWD"
fi"""

[[templates]]
name = "src/main.rs"
template = """fn main() {
    println!("hello {{name}}");
}
"""

[[templates]]
name ="tests/test_main.rs"
template = "// no tests yet for {{name}}"
"#
    .to_string()
}
