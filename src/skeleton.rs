use std::{error::Error, path::PathBuf};

use serde::Deserialize;

use crate::{
    fs_tools::collect_string_from_file,
    template::{template, TemplateArgs},
};

#[derive(Clone, Debug, Deserialize)]
pub struct SkeletonTemplate {
    pub path: String,
    pub template: Option<String>,
    pub include: Option<String>,
}

// a config to deserialize skeleton files in toml
#[derive(Debug, Deserialize)]
pub struct SkeletonConfig {
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub build: Option<String>,
    pub build_first: Option<bool>,
    pub templates: Option<Vec<SkeletonTemplate>>,
}

impl SkeletonConfig {
    // this will iterate over all the given template structs and try and add
    // whatever include point's to, if the include is given the path needs
    // to exists
    pub fn resolve_skeleton_templates(
        &mut self,
        root_path: &str,
        skeleton_name: &str,
        skel_config_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(ref mut temp_files) = self.templates.as_mut() {
            let template_args = TemplateArgs {
                root_path,
                project_name: skeleton_name,
                skel_config_path,
            };

            for template_struct in temp_files.iter_mut() {
                if let Some(include_str) = template_struct.include.as_ref() {
                    let include_path =
                        PathBuf::from(template(&template_args, include_str));

                    let template_string =
                        collect_string_from_file(include_path)?;

                    template_struct.template = Some(template_string);
                } else if template_struct.include.is_none()
                    && template_struct.template.is_none()
                {
                    return Err(Box::from(format!(
                        "entry dose not have a template -- name {} -- path {}",
                        skeleton_name, template_struct.path
                    )));
                }
            }
        }

        Ok(())
    }
}

///! A fully resolved and ready to make skeleton
#[derive(Debug)]
pub struct Skeleton {
    pub name: String,
    pub dirs: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub build: Option<String>,
    pub templates: Option<Vec<SkeletonTemplate>>,
    // these are for template slugs
    pub project_root_string: String,
    pub skel_config_path: String,
    // toggles for the build proses
    pub dont_make_template: bool,
    pub dont_run_build: bool,
    pub build_first: bool,
    pub show_build_output: bool,
}

impl Skeleton {
    pub fn run_template(&self, to_template: &str) -> String {
        let template_args = TemplateArgs {
            root_path: &self.project_root_string,
            project_name: &self.name,
            skel_config_path: &self.skel_config_path,
        };

        template(&template_args, to_template)
    }

    pub fn dir_iter(&self) -> Option<SkeletonPathIterator> {
        match self.dirs.as_ref() {
            Some(dirs) => {
                let template_args = TemplateArgs {
                    root_path: &self.project_root_string,
                    skel_config_path: &self.skel_config_path,
                    project_name: &self.name,
                };

                Some(SkeletonPathIterator::new(template_args, dirs))
            }
            None => None,
        }
    }

    pub fn file_iter(&self) -> Option<SkeletonPathIterator> {
        match self.files.as_ref() {
            Some(files) => {
                let template_args = TemplateArgs {
                    root_path: &self.project_root_string,
                    skel_config_path: &self.skel_config_path,
                    project_name: &self.name,
                };

                Some(SkeletonPathIterator::new(template_args, files))
            }
            None => None,
        }
    }

    pub fn template_iter(&self) -> Option<SkeletonTemplateIterator> {
        match self.templates {
            Some(ref templates) => {
                let template_args = TemplateArgs {
                    root_path: &self.project_root_string,
                    skel_config_path: &self.skel_config_path,
                    project_name: &self.name,
                };

                Some(SkeletonTemplateIterator::new(template_args, templates))
            }
            None => None,
        }
    }
}

pub struct SkeletonPathIterator<'a> {
    // an index counter
    curr: usize,
    max_len: usize,
    // from the skeleton struct
    // to start every path the is not already full / non relative
    root_buf: PathBuf,
    // for templating
    array: &'a Vec<String>,
    template_args: TemplateArgs<'a>,
}

impl<'a> SkeletonPathIterator<'a> {
    pub fn new(
        template_args: TemplateArgs<'a>,
        array: &'a Vec<String>,
    ) -> Self {
        Self {
            root_buf: PathBuf::from(template_args.root_path),
            curr: 0,
            max_len: array.len(),
            array,
            template_args,
        }
    }

    // a wrapper to conveniently call this in the iterator
    fn template(&self, source: &str) -> String {
        template(&self.template_args, source)
    }
}

impl<'a> Iterator for SkeletonPathIterator<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.max_len {
            return None;
        }

        // get and template the current element
        let path_str = self.template(&self.array[self.curr]);

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

pub struct SkeletonTemplateIterator<'a> {
    root_buf: PathBuf,
    array: &'a [SkeletonTemplate],
    max_len: usize,
    curr: usize,
    template_args: TemplateArgs<'a>,
}

impl<'a> SkeletonTemplateIterator<'a> {
    pub fn new(
        template_args: TemplateArgs<'a>,
        array: &'a [SkeletonTemplate],
    ) -> Self {
        Self {
            root_buf: PathBuf::from(template_args.root_path),
            curr: 0,
            max_len: array.len(),
            array,
            template_args,
        }
    }

    fn template(&self, to_temp: &str) -> String {
        template(&self.template_args, to_temp)
    }
}

impl<'a> Iterator for SkeletonTemplateIterator<'a> {
    type Item = (PathBuf, String);

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
                    "WARNING: no template included: \
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

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::{
        make_fake_skel_args, make_fake_skeleton, make_fake_skeleton_config,
    };

    #[test]
    fn test_new_skeleton() {
        let mut config = make_fake_skeleton_config();

        let config_dir = String::from("/tmp/fake_config/config.toml");

        let root = String::from("/tmp/test_path");

        let name = String::from("test_project");

        let args = make_fake_skel_args(&name, "fake_type");

        config
            .resolve_skeleton_templates(&root, &name, &config_dir)
            .expect("cant resolve templates");

        let build_first = if args.build_first
            || (config.build_first.is_some() && config.build_first.unwrap())
        {
            true
        } else {
            false
        };

        let skeleton = Skeleton {
            build_first,
            dirs: config.dirs,
            files: config.files,
            build: config.build,
            templates: config.templates,
            skel_config_path: config_dir,
            name: args.name,
            project_root_string: root,
            dont_make_template: args.dont_make_templates,
            dont_run_build: args.dont_run_build,
            show_build_output: args.show_build_output,
        };

        assert_eq!(skeleton.name, "test_project");

        let test_dirs = ["src", "tests", "tests/more_tests"];

        for d in skeleton.dirs.unwrap() {
            if !test_dirs.contains(&d.as_str()) {
                assert!(false, "{} -- bad test dir found", d);
            }
        }

        let test_files =
            ["src/main.rs", "tests/test_main.rs", "src/test_include.txt"];

        for f in skeleton.files.unwrap() {
            if !test_files.contains(&f.as_str()) {
                assert!(false, "{} -- bad test files found", f);
            }
        }

        // make this test explicitly pass
        assert!(true);
    }

    #[test]
    fn test_dirs_skeleton_buf_iter() {
        let proj = make_fake_skeleton(None);

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
    fn test_files_skeleton_buf_iter() {
        let proj = make_fake_skeleton(None);

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
        let proj = make_fake_skeleton(None);

        let mut template_iter =
            proj.template_iter().expect("cant get template iter");

        // tmp/temp_root/test_project
        let first_test = (
            PathBuf::from("/tmp/test_root/test_project/src/main.rs"),
            "fn main() {\n    println!(\"hello test_project\");\n}\n"
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
    fn test_config_template_iter_no_files() {
        let mut proj = make_fake_skeleton(None);

        proj.files.take();

        assert!(proj.files.is_none(), "did not empty files");

        let mut template_iter = proj.template_iter().expect(
            "cant get template iter in test_config_template_iter_no_files",
        );

        // tmp/temp_root/test_project
        let first_test = (
            PathBuf::from("/tmp/test_root/test_project/src/main.rs"),
            "fn main() {\n    println!(\"hello test_project\");\n}\n"
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
