use std::{env, fs, path::PathBuf};

use tempfile::{tempdir, TempDir};

use crate::parse_args::SkelArgs;

pub const TEST_PROJECT_KEY: &str = "test_project";
pub const TEST_PROJECT_PATH: &str = "{{config_dir}}/projects/test_project.toml";
pub static TEST_PROJECT_ALIASES: &[&str] = &["t", "T", "test_project"];

pub const TEST_CONFIG: &str = r#"
[skeletons]
test_project.path = "{{config_dir}}/projects/test_project.toml"
test_project.aliases = ["t", "T", "test_project"]
"#;

pub const TEST_SKEL_NAME: &str = "test_project.toml";

pub const TEST_SKEL: &str = r#"
dirs = [
    "test_src",
    "another_test_dir"
]

files = [
    "test_src/test_main.rs",
    "another_test_files.txt",
]

[[templates]]
path = "test_src/test_template_file.txt"
template = "this is a test template for {{name}}"

[[templates]]
path = "test_src/test_template_include_file.txt"
include = "{{config_dir}}/projects/test_include_file.txt"
"#;

pub const TEST_INCLUDE_NAME: &str = "projects/test_include_file.txt";

pub const TEST_INCLUDE_STR: &str = "this is the include test file for {{name}}";

pub struct TestData {
    pub temp_dir: TempDir,
    pub temp_path: PathBuf,
    pub temp_path_string: String,
    pub test_main_config_path: Option<PathBuf>,
}

impl Default for TestData {
    fn default() -> Self {
        let temp_dir = tempdir().expect("could not make temp directory");

        let temp_path = temp_dir.path().to_owned();

        assert!(temp_path.exists(), "could not make temp directory");

        let temp_path_string = temp_path
            .as_os_str()
            .to_str()
            .expect("cant get temp path string")
            .to_owned();

        Self {
            temp_dir,
            temp_path,
            temp_path_string,
            test_main_config_path: None,
        }
    }
}

impl TestData {
    pub fn make_configs(&mut self) {
        let mut config_path = self.temp_path.clone();

        config_path.push("skel");

        fs::create_dir_all(&config_path)
            .expect("could not make test config dir");

        config_path.push("config.toml");

        fs::write(&config_path, TEST_CONFIG)
            .expect("was not able to make test config");

        self.test_main_config_path = Some(config_path);

        let mut projects_path = self.temp_path.clone();

        projects_path.push("projects");

        fs::create_dir_all(projects_path).unwrap();

        let mut include_file = self.temp_path.clone();

        include_file.push(TEST_INCLUDE_NAME);

        fs::write(include_file, TEST_INCLUDE_STR)
            .expect("was not able to make TEST_INCLUDE_STR");

        let mut test_skeleton = self.temp_path.clone();

        test_skeleton.push("projects");

        test_skeleton.push(TEST_SKEL_NAME);

        fs::write(test_skeleton, TEST_SKEL)
            .expect("was not able to make the test skeleton");

        env::set_var("XDG_CONFIG_HOME", &self.temp_path_string);
    }
}

pub fn test_args() -> SkelArgs {
    SkelArgs {
        skeleton: Some(String::from("t")),
        name: String::from("test_project"),
        dry_run: true,
        ..Default::default()
    }
}
