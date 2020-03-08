#![allow(dead_code)]
use std::{error::Error, fs, path::PathBuf};

use tempfile::{tempdir, TempDir};

use toml;

use crate::{
    cli::{SkelArgs, UserConfig},
    project::{Project, ProjectConfig},
};

#[derive(Default)]
pub struct TempSetup {
    pub root: PathBuf,
    pub temp: Option<TempDir>,
    pub project: Option<Project>,
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

    pub fn make_fake_user_config(&self) -> Result<(), Box<dyn Error>> {
        use std::io::Write;

        let mut fake_config = self.root_buf();

        fake_config.push("fake_skel");

        fs::create_dir_all(&fake_config).expect("cant make config dir");

        fake_config.push("fake_config.toml");

        let mut config_file = fs::File::create(fake_config)?;

        let toml_str = make_fake_user_toml();

        config_file.write_all(toml_str.as_bytes())?;

        Ok(())
    }

    pub fn make_fake_include(&self) -> Result<(), Box<dyn Error>> {
        use std::io::Write;

        let mut fake_inlcude = self.root_buf();

        fake_inlcude.push("test_include");

        let mut include_file = fs::File::create(fake_inlcude)?;

        include_file.write_all(b"test include value")?;

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

pub fn make_fake_project_config() -> ProjectConfig {
    let fake_toml = make_fake_project_toml();

    let mut conf = toml::from_str::<ProjectConfig>(&fake_toml)
        .expect("cant make config from fake toml");

    conf.config_dir_string = Some(String::from("/tmp/fake_config.toml"));

    conf
}

pub fn make_fake_project(root: Option<PathBuf>) -> Project {
    let mut root: String = if let Some(root) = root {
        String::from(root.to_str().expect("cant get temp path a str"))
    } else {
        String::from("/tmp/test_root")
    };

    let mut conf = make_fake_project_config();

    conf.resolve_files();

    let name = String::from("test_project");

    root.push('/');
    root.push_str(&name);

    let args = SkelArgs::make_fake(&name, "fake_type");

    Project::new(root, &args, conf)
}

pub fn make_fake_user_config() -> UserConfig {
    let fake_toml = make_fake_user_toml();

    toml::from_str::<UserConfig>(&fake_toml)
        .expect("did not make user config from toml")
}

pub fn make_fake_user_toml() -> String {
    r#"
# the paths to the projects
# {{config-dir}} will correspond to ~/.config/skel
[projects]
basic_cpp = "{{config-dir}}/projects/basic_cpp.toml"
basic_javascript = "{{config-dir}}/projects/basic_javascript.toml"
basic_python = "{{config-dir}}/projects/basic_python.toml"

# alias's to use on the cli
[alias]
basic_cpp = ["cpp", "cp", "c++"]
basic_javascript = ["js", "j"]
basic_python = ["py", "p"]
        "#
    .to_owned()
}

pub fn make_fake_project_toml() -> String {
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
path = "src/main.rs"
template = """fn main() {
    println!("hello {{name}}");
}
"""

[[templates]]
path ="tests/test_main.rs"
template = "// no tests yet for {{name}}"

[[templates]]
path = "src/test_include"
include = "docs/test_include.txt"
"#
    .to_string()
}
