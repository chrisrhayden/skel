use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use toml;

use crate::{
    cli::SkelArgs, fs_tools::collect_string_from_file, template::template,
};

// a config to deserialize project files in toml
#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub build: Option<String>,
    pub templates: Option<Vec<FileTemplate>>,
}

#[derive(Debug, Deserialize)]
pub struct FileTemplate {
    pub path: String,
    pub template: Option<String>,
    pub include: Option<String>,
}

// a project struct to hold the project build data
// the root, current_dir + given on cli plus name, i.e. pwd/project_name
#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub build: Option<String>,
    pub templates: Option<Vec<FileTemplate>>,
    pub root_path: PathBuf,
    // these are for template slugs
    pub root_string: String,
    pub config_dir_string: String,
    // toggles for the build proses
    pub dont_make_template: bool,
    pub dont_run_build: bool,
}

impl Project {
    pub fn new(
        root: String,
        config_dir_string: String,
        args: &SkelArgs,
        config: ProjectConfig,
    ) -> Self {
        Self {
            name: args.name.to_owned(),
            root_path: PathBuf::from(&root),
            root_string: root,
            config_dir_string,
            files: config.files,
            dirs: config.dirs,
            build: config.build,
            templates: config.templates,
            dont_make_template: args.dont_make_templates,
            dont_run_build: args.dont_run_build,
        }
    }

    pub fn template_str(&self, to_template: &str) -> String {
        template(
            &self.root_string,
            &self.name,
            &self.config_dir_string,
            to_template,
        )
    }

    pub fn resolve_templates(&mut self) -> Result<(), Box<dyn Error>> {
        if self.templates.is_none() {
            return Ok(());
        }

        let mut errors: Vec<String> = Vec::new();

        for template_struct in self.templates.as_mut().unwrap() {
            // the include variable is present force the template to whatever
            // is in the include path if it exists
            if let Some(include_str) = template_struct.include.as_ref() {
                let include_path = PathBuf::from(template(
                    &self.config_dir_string,
                    &self.name,
                    &self.config_dir_string,
                    include_str,
                ));

                template_struct.template =
                    Some(collect_string_from_file(include_path)?);
            } else if template_struct.include.is_none()
                && template_struct.template.is_none()
            {
                errors.push(template_struct.path.to_owned());
            }
        }

        if !errors.is_empty() {
            let error_string = format!(
                "template['s] missing include path or template string -- {}",
                errors.join(", ")
            );

            Err(Box::from(error_string))
        } else {
            Ok(())
        }
    }

    pub fn root_string(&self) -> String {
        self.root_path
            .to_str()
            .expect("cant get root string")
            .to_owned()
    }

    pub fn dir_iter(&self) -> Option<ProjectPathIterator> {
        if let Some(dirs) = self.dirs.as_ref() {
            let root =
                self.root_path.to_str().expect("cant covert path to str");

            Some(ProjectPathIterator::new(
                root,
                &self.name,
                &self.config_dir_string,
                dirs,
            ))
        } else {
            None
        }
    }

    pub fn file_iter(&self) -> Option<ProjectPathIterator> {
        if let Some(files) = self.files.as_ref() {
            let root =
                self.root_path.to_str().expect("cant covert path to str");

            Some(ProjectPathIterator::new(
                root,
                &self.name,
                &self.config_dir_string,
                files,
            ))
        } else {
            None
        }
    }

    pub fn template_iter(&self) -> Option<ProjectTemplateIterator> {
        let root = self.root_path.to_str().expect("cant covert path to str");

        match &self.templates {
            Some(templates) => Some(ProjectTemplateIterator::new(
                root,
                &self.name,
                &self.config_dir_string,
                &templates,
            )),
            None => None,
        }
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
    conf: &'a str,
    array: &'a [String],
}

impl<'a> ProjectPathIterator<'a> {
    pub fn new(
        root: &'a str,
        name: &'a str,
        conf: &'a str,
        array: &'a [String],
    ) -> Self {
        Self {
            root,
            conf,
            root_buf: PathBuf::from(root),
            name,
            curr: 0,
            max_len: array.len(),
            array,
        }
    }

    fn template(&self, source: &str) -> String {
        template(self.root, self.name, self.conf, source)
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

pub struct ProjectTemplateIterator<'a> {
    root_buf: PathBuf,
    root: &'a str,
    name: &'a str,
    conf: &'a str,
    array: &'a [FileTemplate],
    max_len: usize,
    curr: usize,
}

impl<'a> ProjectTemplateIterator<'a> {
    pub fn new(
        root: &'a str,
        name: &'a str,
        conf: &'a str,
        array: &'a [FileTemplate],
    ) -> Self {
        Self {
            root,
            root_buf: PathBuf::from(root),
            name,
            conf,
            curr: 0,
            max_len: array.len(),
            array,
        }
    }

    fn template(&self, to_temp: &str) -> String {
        template(self.root, self.name, self.conf, to_temp)
    }
}

impl<'a> Iterator for ProjectTemplateIterator<'a> {
    type Item = (PathBuf, String);

    // there should be very little trouble here, maybe do some checks as we take
    // in data from ProjectConfig
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.max_len {
            return None;
        }

        let next_to_template = &self.array[self.curr];

        self.curr += 1;

        // NOTE: idk if i want a missing template to be an error or not
        let template_string = if next_to_template.template.is_some() {
            self.template(next_to_template.template.as_ref().unwrap())
        } else {
            String::new()
        };

        let path_name = self.template(&next_to_template.path);

        let mut path_buf = self.root_buf.clone();

        path_buf.push(path_name);

        Some((path_buf, template_string))
    }
}

// return a config from a toml file
pub fn collect_project_config(
    path: &Path,
) -> Result<ProjectConfig, Box<dyn Error>> {
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

    use crate::test_utils::{
        make_fake_project, make_fake_project_config, make_fake_project_toml,
        TempSetup,
    };

    fn make_fake_conifg_file(root: &Path) -> bool {
        use std::io::Write;

        let toml_str = make_fake_project_toml();
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

        match collect_project_config(&fake_path) {
            Err(err) => assert!(false, "{} bad toml config", err),
            Ok(config) => {
                assert_eq!(
                    config.dirs.as_ref().unwrap()[0],
                    String::from("src"),
                    "did not get the right name"
                );

                for entry in config.dirs.as_ref().unwrap().iter() {
                    assert_eq!(entry.is_empty(), false, "no dirs in array");
                }
            }
        };
    }

    #[test]
    fn test_new_project() {
        let config = make_fake_project_config();
        let config_dir = String::from("/tmp/fake_config/config.toml");

        let root = String::from("/tmp/test_path");

        let name = String::from("test_project");

        let args = SkelArgs::make_fake(&name, "fake_type");

        let project = Project::new(root, config_dir, &args, config);

        assert_eq!(project.name, "test_project");

        for d in project.dirs.unwrap() {
            if d != "src" && d != "tests" && d != "tests/more_tests" {
                assert!(false, "{} -- test dirs not found", d);
            }
        }

        for f in project.files.unwrap() {
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

        let mut dir_iter = proj.dir_iter().unwrap();

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

        let mut file_iter = proj.file_iter().unwrap();

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

    #[test]
    fn test_config_template_iter() {
        let proj = make_fake_project(None);

        let mut template_iter =
            proj.template_iter().expect("cant get template iter");

        // tmp/temp_root/test_project
        let first_test = (
            PathBuf::from("/tmp/test_root/test_project/src/main.rs"),
            r#"fn main() {
    println!("hello test_project");
}
"#
            .to_string(),
        );

        let first = template_iter
            .next()
            .expect("failed to call next on template_iter");

        assert_eq!(first.0, first_test.0, "first path is not the same");
        assert_eq!(first.1, first_test.1, "first string is not the same");

        let second_test = (
            PathBuf::from("/tmp/test_root/test_project/tests/test_main.rs"),
            String::from("// no tests yet for test_project"),
        );

        let second = template_iter
            .next()
            .expect("failed to call next on template_iter");

        assert_eq!(second.0, second_test.0, "second path is not the same");
        assert_eq!(second.1, second_test.1, "second string is not the same");
    }

    #[test]
    fn test_resolve_templates() {
        let mut temp = TempSetup::default();
        temp.setup();
        temp.make_fake_include().expect("cant make include file");

        let project = temp.project.as_mut().unwrap();

        match project.resolve_templates() {
            Ok(_) => assert!(true),
            Err(err) => {
                assert!(false, "{}", err);
            }
        };

        let mut temp_iter = project.template_iter().unwrap();

        let (_, include_str) = temp_iter.nth(2).unwrap();

        assert_eq!(
            include_str, "test include value\n",
            "include values dont match"
        );
    }
}
