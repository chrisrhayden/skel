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
pub struct ProjectConfigFile {
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub build: Option<String>,
    pub templates: Option<Vec<ProjectTemplate>>,
}

impl ProjectConfigFile {
    pub fn resolve_project_templates(
        &mut self,
        root: &str,
        name: &str,
        config_dir_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        // add templates or include files to files list
        if let Some(ref mut temp_files) = self.templates.take() {
            for template_struct in temp_files.iter_mut() {
                // the include variable is present force the template to whatever
                // is in the include path if it exists
                if let Some(include_str) = template_struct.include.as_ref() {
                    let include_path = PathBuf::from(template(
                        root,
                        name,
                        config_dir_path,
                        include_str,
                    ));

                    let template_string =
                        collect_string_from_file(include_path)?;

                    template_struct.template = Some(template_string);
                } else if template_struct.include.is_none()
                    && template_struct.template.is_none()
                {
                    return Err(Box::from(format!(
                        "entry dose not have a template -- name {} -- path {}",
                        name, template_struct.path
                    )));
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectTemplate {
    pub path: String,
    pub template: Option<String>,
    pub include: Option<String>,
}

pub struct ProjectArgs {
    pub files: Option<Vec<String>>,
    pub dirs: Option<Vec<String>>,
    pub root: String,
    pub build: Option<String>,
    pub templates: Option<Vec<ProjectTemplate>>,
    pub config_dir_string: String,
    pub args: SkelArgs,
}

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
<<<<<<< HEAD
    // toggles for the build proses
=======
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub build: Option<String>,
    pub templates: Option<Vec<ProjectTemplate>>,
>>>>>>> dev
    pub dont_make_template: bool,
    pub dont_run_build: bool,
}

impl Project {
<<<<<<< HEAD
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
=======
    pub fn new(project_args: ProjectArgs) -> Self {
        Self {
            files: project_args.files.clone(),
            dirs: project_args.dirs,
            name: project_args.args.name.to_owned(),
            root_path: PathBuf::from(&project_args.root),
            root_string: project_args.root,
            config_dir_string: project_args.config_dir_string,
            build: project_args.build,
            templates: project_args.templates.clone(),
            dont_make_template: project_args.args.dont_make_templates,
            dont_run_build: project_args.args.dont_run_build,
>>>>>>> dev
        }
    }

    pub fn run_template(&self, to_template: &str) -> String {
        template(
            &self.root_string,
            &self.name,
            &self.config_dir_string,
            to_template,
        )
    }

    pub fn root_string(&self) -> String {
        self.root_path
            .to_str()
            .expect("cant get root string")
            .to_owned()
    }

    pub fn dir_iter(&self) -> Option<ProjectPathIterator> {
<<<<<<< HEAD
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
=======
        if self.dirs.is_none() {
            return None;
        }

        Some(ProjectPathIterator::new(
            &self.root_string,
            &self.name,
            &self.config_dir_string,
            self.dirs
                .as_ref()
                .unwrap()
                .iter()
                .map(|ele| ele.as_str())
                .collect::<Vec<&str>>(),
        ))
    }

    pub fn file_iter(&self) -> Option<ProjectPathIterator> {
        if self.files.is_none() {
            return None;
        }

        let mut files = self
            .files
            .as_ref()
            .unwrap()
            .iter()
            .map(|ele| ele.as_str())
            .collect::<Vec<&str>>();

        if let Some(templates) = self.templates.as_ref() {
            let mut temps = templates
                .iter()
                .map(|te| te.path.as_str())
                .collect::<Vec<&str>>();

            files.append(&mut temps);
        };

        Some(ProjectPathIterator::new(
            &self.root_string,
            &self.name,
            &self.config_dir_string,
            files,
        ))
>>>>>>> dev
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

// an Iterator that takes an array of strings
// then templates the string and returns a PathBuf
// TODO: the templating is bad and I feel bad
pub struct ProjectPathIterator<'a> {
    // an index counter
    curr: usize,
    max_len: usize,
    // from the project struct
    root: &'a str,
    // to start every path the is not already full / non relative
    root_buf: PathBuf,
    // for templating
    name: &'a str,
    conf: &'a str,
    array: Vec<&'a str>,
}

impl<'a> ProjectPathIterator<'a> {
    ///! new takes the template slug keys and the collection to iterate over
    pub fn new(
        root: &'a str,
        name: &'a str,
        conf: &'a str,
        array: Vec<&'a str>,
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

    // a wrapper to conveniently call this in the iterator
    fn template(&self, source: &str) -> String {
        template(self.root, self.name, self.conf, source)
    }
}

impl<'a> Iterator for ProjectPathIterator<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.max_len {
            return None;
        }

        // get and template the current element
        let path_str = self.template(self.array[self.curr]);

        // go to the next element
        self.curr += 1;

        // if the path is not yet full add the root str
        let path_buf = if path_str.starts_with('/') {
            PathBuf::from(path_str)
        } else {
            let mut path_buf = self.root_buf.clone();
            path_buf.push(path_str);

            path_buf
        };

        // when the path is finally used it should be done in full to avoid
        // weird errors with relative path's
        assert!(path_buf.starts_with("/"), "BUG: path was made wrong");

        Some(path_buf)
    }
}

pub struct ProjectTemplateIterator<'a> {
    root_buf: PathBuf,
    root: &'a str,
    name: &'a str,
    conf: &'a str,
    array: &'a [ProjectTemplate],
    max_len: usize,
    curr: usize,
}

impl<'a> ProjectTemplateIterator<'a> {
    ///! new takes the template slug keys and the collection to iterate over
    pub fn new(
        root: &'a str,
        name: &'a str,
        conf: &'a str,
        array: &'a [ProjectTemplate],
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

        let template_string =
            if let Some(template_str) = next_to_template.template.as_ref() {
                self.template(template_str)
            } else {
                eprintln!(
                    "WARNING: no template included\n\
                    pleas add the necessary variables to the config"
                );

                String::new()
            };

        let path_str = self.template(&next_to_template.path);

        let path_buf = if path_str.starts_with('/') {
            PathBuf::from(path_str)
        } else {
            let mut path_buf = self.root_buf.clone();
            path_buf.push(path_str);

            path_buf
        };

        assert!(path_buf.starts_with("/"), "BUG: path was made wrong");

        Some((path_buf, template_string))
    }
}

// return a config from a toml file
pub fn collect_project_config(
    path: &Path,
) -> Result<ProjectConfigFile, Box<dyn Error>> {
    use std::io::Read;

    // return if project path dose not exists
    if !path.exists() {
        return Err(Box::from(format!("path given dose exists -- {:?}", path)));
    }

    let mut dir_config = fs::File::open(path)?;

    let mut buf = String::new();

    dir_config.read_to_string(&mut buf)?;

    let config = match toml::from_str::<ProjectConfigFile>(&buf) {
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

        let proj_args = ProjectArgs {
            root,
            args,
            dirs: config.dirs,
            build: config.build,
            files: config.files,
            templates: config.templates,
            config_dir_string: config_dir,
        };

        let project = Project::new(proj_args);

        assert_eq!(project.name, "test_project");

<<<<<<< HEAD
        for d in project.dirs.unwrap() {
            if d != "src" && d != "tests" && d != "tests/more_tests" {
                assert!(false, "{} -- test dirs not found", d);
            }
        }

        for f in project.files.unwrap() {
            if f != "src/main.rs" && f != "tests/test_main.rs" {
                assert!(false, "{} -- test files not found", f);
=======
        let test_dirs = ["src", "tests", "tests/more_tests"];

        for d in project.dirs.unwrap() {
            if !test_dirs.contains(&d.as_str()) {
                assert!(false, "{} -- bad test dir found", d);
            }
        }

        let test_files =
            ["src/main.rs", "tests/test_main.rs", "src/test_include.txt"];

        for f in project.files.unwrap() {
            if !test_files.contains(&f.as_str()) {
                assert!(false, "{} -- bad test files found", f);
>>>>>>> dev
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
}
