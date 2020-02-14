use std::error::Error;
use std::fs;
use std::path::{PathBuf, Path};
use std::env;

use toml;
use serde::Deserialize;

use crate::template::template;
#[derive(Debug, Deserialize)]
pub struct Template {
    pub name: String,
    pub template: String,
}

// a config to deserialize project files in toml
#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub dirs: Vec<String>,
    pub files: Vec<String>,
    pub build: Option<String>,
    pub templates: Vec<Template>,
}

// a project struct to eventually use to make the project
// last should take precedent:
// default > file config > cli config (this takes precedent)
#[derive(Debug)]
pub struct Project {
    // the root, current_dir/given on cli plus name
    pub root: PathBuf,
    // the project name
    pub name: String,
    // a vec of directory's to make
    pub dirs: Vec<String>,
    // a vec of file's to make
    pub files: Vec<String>,
    // a build script to make and run
    pub build: Option<String>,
}

impl Project {
    pub fn new(
        cli_root: Option<String>,
        cli_name: Option<String>,
        config: ProjectConfig,
    ) -> Self {
        // this is kind of weird
        let mut root = if let Some(cli_root) = cli_root {
            PathBuf::from(cli_root)
        } else {
            env::current_dir().expect("cant get current dir")
        };

        // set the cli if any
        let name = if let Some(name) = cli_name {
            name
        } else if let Some(name) = config.name {
            name
        } else {
            panic!("no name for project, wtf");
        };

        // set root to the project name not the current_dir
        // or the one given on the cli
        root.push(&name);

        Self {
            root,
            name,
            dirs: config.dirs,
            files: config.files,
            build: config.build,
        }
    }

    pub fn dir_iter(&self) -> ProjectPathIterator {
        let root = self
            .root
            .as_os_str()
            .to_str()
            .expect("cant covert path to str");

        ProjectPathIterator::new(root, &self.name, &self.dirs)
    }

    pub fn file_iter(&self) -> ProjectPathIterator {
        let root = self
            .root
            .as_os_str()
            .to_str()
            .expect("cant covert path to str");

        ProjectPathIterator::new(root, &self.name, &self.files)
    }
}

// an Iterator that returns a path buff fro each file given in the array
// TODO: the templating is bad and I feel bad
pub struct ProjectPathIterator<'a> {
    curr: usize,
    max_len: usize,
    // from the project struct
    root: &'a str,
    root_buf: PathBuf,
    name: &'a str,
    array: &'a [String],
}

impl<'a> ProjectPathIterator<'a> {
    pub fn new(root: &'a str, name: &'a str, array: &'a [String]) -> Self {
        Self {
            root,
            root_buf: PathBuf::from(root),
            name,
            curr: 0,
            max_len: array.len(),
            array,
        }
    }

    fn template(&self, source: &str) -> String {
        template(self.root, self.name, source)
    }
}

impl<'a> Iterator for ProjectPathIterator<'a> {
    type Item = PathBuf;

    // there should be very little trouble here, maybe do some checks as we take
    // in data from ProjectConfig
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.max_len {
            return None;
        }

        let next_str = &self.array[self.curr];

        self.curr += 1;

        let path_str = self.template(next_str);

        let mut path_buf = self.root_buf.clone();
        path_buf.push(path_str);

        Some(path_buf)
    }
}

// return a config from a toml file
pub fn collect_config(path: &Path) -> Result<ProjectConfig, Box<dyn Error>> {
    use std::io::Read;

    let mut dir_config = fs::File::open(path)?;
    let mut buf = String::new();

    dir_config.read_to_string(&mut buf)?;

    let config = match toml::from_str::<ProjectConfig>(&buf) {
        Ok(val) => val,
        Err(err) => {
            return Err(Box::from(format!(
                "Toml Error in project file: {}",
                err
            )))
        }
    };

    Ok(config)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::{make_fake_project, TempSetup, make_fake_config};

    pub fn make_fake_toml() -> String {
        r#"
        name = "test_project"

        dirs = [
            "src",
            "tests",
            "tests/fuck"
        ]
        files = [
            "src/main.rs",
            "tests/test_main.rs"
        ]

        [[templates]]
        name = "src/main.ss"
        template = """fn main() {
            println!("hello world");
        }
        """

        [[templates]]
        name ="tests/test_main.rs"
        template = "// no tests yet"
        "#
        .to_string()
    }

    fn make_fake_conifg_file(root: &Path) -> bool {
        use std::io::Write;

        let toml_str = make_fake_toml();
        let mut fake_conf =
            fs::File::create(root).expect("cant make file in temp");

        fake_conf
            .write_all(toml_str.as_bytes())
            .expect("cant write to fake toml file");

        true
    }

    #[test]
    fn test_collect_config() {
        let mut temp = TempSetup::default();
        let mut fake_path = temp.setup();

        fake_path.push("fake_project.toml");

        if !make_fake_conifg_file(&fake_path) {
            assert!(false, "failed to make fake config in temp dir");
        }

        match collect_config(&fake_path) {
            Err(err) => assert!(false, "{} bad toml config", err),
            Ok(config) => {
                println!("{:#?}", config);
                assert_eq!(config.name, Some(String::from("test_project")));
                assert!(config.dirs.is_empty() == false, "config dirs empty");

                for entry in config.dirs {
                    assert!(entry.is_empty() == false, "no dirs in array");
                }
            }
        };
    }

    #[test]
    fn test_new_project() {
        let config = make_fake_config();

        let root = Some(String::from("/tmp/test_path"));

        let name = Some(String::from("test_project"));

        let project = Project::new(root, name, config);

        assert_eq!(project.name, "test_project");

        for d in project.dirs {
            if d != "src" && d != "tests" && d != "tests/more_tests" {
                assert!(false, "{} -- test dirs not found", d);
            }
        }

        for f in project.files {
            if f != "src/main.rs" && f != "tests/test_main.rs" {
                assert!(false, "{} -- test files not found", f);
            }
        }

        // make this test explicitly pass
        assert!(true);
    }

    #[test]
    fn test_dirs_project_buf_iter() {
        let proj = make_fake_project(None);

        let mut dir_iter = proj.dir_iter();

        assert_eq!(
            dir_iter.next(),
            Some(PathBuf::from("/tmp/test_root/test_project/src")),
            "src not in project struct"
        );

        assert_eq!(
            dir_iter.next(),
            Some(PathBuf::from("/tmp/test_root/test_project/tests")),
            "tests not in project struct"
        );

        assert_eq!(
            dir_iter.next(),
            Some(PathBuf::from(
                "/tmp/test_root/test_project/tests/more_tests"
            )),
            "tests not in project struct"
        );

        assert_eq!(dir_iter.next(), None, "too many in dirs vector");

        assert!(true);
    }

    #[test]
    fn test_files_project_buf_iter() {
        let proj = make_fake_project(None);

        let mut file_iter = proj.file_iter();

        assert_eq!(
            file_iter.next(),
            Some(PathBuf::from("/tmp/test_root/test_project/src/main.rs"),),
            "main not in project struct"
        );

        assert_eq!(
            file_iter.next(),
            Some(PathBuf::from(
                "/tmp/test_root/test_project/tests/test_main.rs"
            )),
            "test_main.rs not in project struct"
        );

        assert!(true);
    }
}
