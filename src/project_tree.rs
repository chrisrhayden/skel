use std::{
    collections::HashSet,
    error::Error,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::Command,
};

use crate::config::{RunConfig, SkelTemplate};

// the derives are to be used in the HashSet
#[derive(std::cmp::Eq, std::cmp::PartialEq, std::hash::Hash)]
struct TemplateFile {
    path: PathBuf,
    template: String,
}

struct SkelTree<'a> {
    templates: Option<HashSet<TemplateFile>>,
    files: Option<HashSet<PathBuf>>,
    dirs: Option<HashSet<PathBuf>>,
    build: Option<&'a str>,
    build_first: bool,
}

fn run_build_script(build: &str) {
    let mut build_script = String::from("#!/usr/bin/bash env\n\n");

    build_script.push_str(build);

    let output = Command::new("bash")
        .arg("-c")
        .arg(build_script)
        .output()
        .expect("build error");

    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

// collect the template into its own struct
//
// this will render the include file
fn resolved_template(
    skel_template: &SkelTemplate,
    run_conf: &RunConfig,
) -> Result<TemplateFile, Box<dyn Error>> {
    let template = if let Some(include) = skel_template.include.as_ref() {
        let template_file_string = match fs::read_to_string(&include) {
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    return Err(Box::from(format!(
                        "include file not found {}",
                        include
                    )));
                }
                _ => return Err(Box::from(err)),
            },
            Ok(value) => value,
        };

        run_conf
            .handle
            .render_template(&template_file_string, &run_conf.template_data)?
    } else if let Some(template_str) = &skel_template.template {
        template_str.clone()
    } else {
        return Err(Box::from(String::from(
            "no template string or include path for template",
        )));
    };

    let mut path = run_conf.root_path.clone();

    path.push(&skel_template.path);

    let new_template = TemplateFile { path, template };

    Ok(new_template)
}

// resolve all the templates and add them to a hash set
fn resolve_templates(
    run_conf: &RunConfig,
) -> Result<Option<HashSet<TemplateFile>>, Box<dyn Error>> {
    if let Some(templates) = run_conf.skel_conf.templates.as_ref() {
        let mut resolved_templates = HashSet::new();
        for template in templates {
            let new_template = resolved_template(template, run_conf)?;

            resolved_templates.insert(new_template);
        }
        Ok(Some(resolved_templates))
    } else {
        Ok(None)
    }
}

// add all the dirs to a hash set
fn resolve_dirs(
    run_conf: &RunConfig,
) -> Result<Option<HashSet<PathBuf>>, Box<dyn Error>> {
    if let Some(dirs) = run_conf.skel_conf.dirs.as_ref() {
        let mut resolved_dirs = HashSet::new();

        for dir in dirs {
            let mut dir_path = run_conf.root_path.clone();

            dir_path.push(&dir);

            resolved_dirs.insert(dir_path);
        }

        Ok(Some(resolved_dirs))
    } else {
        Ok(None)
    }
}

type FilesResult =
    Result<Option<(Vec<PathBuf>, HashSet<PathBuf>)>, Box<dyn Error>>;

// add all the files to a hash set and return the set along with a vec of the
// parent dirs
//
// the parent dirs will be added to the dir hash set
fn resolve_files(run_conf: &RunConfig) -> FilesResult {
    if let Some(files) = run_conf.skel_conf.files.as_ref() {
        let mut resolved_files = HashSet::new();
        let mut resolved_dirs = vec![];

        for file in files {
            let mut file_path = run_conf.root_path.clone();

            file_path.push(&file);

            // NOTE: this is probably fine as we have pushed the project root
            // dir first
            let parent = file_path.parent().unwrap().to_owned();

            resolved_files.insert(file_path);
            resolved_dirs.push(parent);
        }
        Ok(Some((resolved_dirs, resolved_files)))
    } else {
        Ok(None)
    }
}

fn make_tree(skel_tree: &SkelTree) -> Result<(), Box<dyn Error>> {
    if skel_tree.build.is_some() && skel_tree.build_first {
        run_build_script(skel_tree.build.unwrap());
    }

    if let Some(ref dirs) = skel_tree.dirs {
        for dir in dirs {
            fs::create_dir_all(dir)?;
        }
    }

    if let Some(ref files) = skel_tree.files {
        for file in files {
            fs::File::create(file)?;
        }
    }

    if let Some(ref templates) = skel_tree.templates {
        for template in templates {
            fs::write(&template.path, &template.template)?;
        }
    }

    if skel_tree.build.is_some() && !skel_tree.build_first {
        run_build_script(skel_tree.build.unwrap());
    }

    Ok(())
}

fn print_tree(root: &Path, skel_tree: &SkelTree) {
    if root.exists() {
        println!(
            "\x1b[33mWarning {} already exists\x1b[0m\n",
            root.as_os_str().to_str().unwrap()
        );
    }

    println!("would make in to -> {}", root.as_os_str().to_str().unwrap());

    if let Some(ref dirs) = skel_tree.dirs {
        for dir in dirs {
            println!("  dir  -> {}", dir.as_os_str().to_str().unwrap());
        }
    }

    if let Some(ref files) = skel_tree.files {
        for file in files {
            println!("  file -> {}", file.as_os_str().to_str().unwrap());
        }
    }

    if let Some(ref templates) = skel_tree.templates {
        for template in templates {
            println!("  ------");
            println!(
                "  template -> {}",
                template.path.as_os_str().to_str().unwrap()
            );

            for line in template.template.lines() {
                println!("    {}", line);
            }

            println!("  ------");
        }
    }

    if let Some(build) = skel_tree.build {
        println!("  build first = {}", skel_tree.build_first);

        println!("  ------\n    {}\n  ------", build);
    }
}

// collect all the parts of the skeleton in to hash sets
//
// the hash sets are to make sure there are no duplicates, this is mostly for
// printing out the dry run
pub fn make_project_tree(
    dry_run: bool,
    run_conf: &RunConfig,
) -> Result<(), Box<dyn Error>> {
    let templates = resolve_templates(run_conf)?;

    let mut dirs = resolve_dirs(run_conf)?;

    let files = match resolve_files(run_conf)? {
        None => None,
        Some((parent_dirs, files)) => {
            if let Some(ref mut dirs) = dirs {
                for dir in parent_dirs.into_iter() {
                    dirs.insert(dir);
                }
            } else {
                dirs =
                    Some(parent_dirs.into_iter().collect::<HashSet<PathBuf>>());
            }

            Some(files)
        }
    };

    let build_first = run_conf.skel_conf.build_first.unwrap_or(false);

    let skel_tree = SkelTree {
        files,
        dirs,
        templates,
        build_first,
        build: run_conf.skel_conf.build.as_deref(),
    };

    if dry_run {
        print_tree(&run_conf.root_path, &skel_tree);

        Ok(())
    } else {
        make_tree(&skel_tree)
    }
}
