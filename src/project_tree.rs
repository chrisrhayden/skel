use std::{
    collections::HashSet, error::Error, fs, io::ErrorKind, path::PathBuf,
};

use crate::{
    config::{RunConfig, SkelTemplate},
    parse_args::SkelArgs,
};

#[derive(std::cmp::Eq, std::cmp::PartialEq, std::hash::Hash)]
struct TemplateFile {
    path: PathBuf,
    template: String,
}

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

fn resolve_templates(
    run_conf: &RunConfig,
) -> Result<HashSet<TemplateFile>, Box<dyn Error>> {
    let mut resolved_templates = HashSet::new();

    if let Some(templates) = run_conf.skel_conf.templates.as_ref() {
        for template in templates {
            let new_template = resolved_template(template, run_conf)?;

            resolved_templates.insert(new_template);
        }
    }

    Ok(resolved_templates)
}

fn resolve_dirs(
    run_conf: &RunConfig,
) -> Result<HashSet<PathBuf>, Box<dyn Error>> {
    let mut resolved_dirs = HashSet::new();

    if let Some(dirs) = run_conf.skel_conf.dirs.as_ref() {
        for dir in dirs {
            let mut dir_path = run_conf.root_path.clone();

            dir_path.push(&dir);

            resolved_dirs.insert(dir_path);
        }
    }

    Ok(resolved_dirs)
}

fn resolve_files(
    run_conf: &RunConfig,
) -> Result<(Vec<PathBuf>, HashSet<PathBuf>), Box<dyn Error>> {
    let mut resolved_files = HashSet::new();
    let mut resolved_dirs = vec![];

    if let Some(files) = run_conf.skel_conf.files.as_ref() {
        for file in files {
            let mut file_path = run_conf.root_path.clone();

            file_path.push(&file);

            // NOTE: this is probably fine as we have pushed the project root
            // dir first
            let parent = file_path.parent().unwrap().to_owned();

            resolved_files.insert(file_path);
            resolved_dirs.push(parent);
        }
    }

    Ok((resolved_dirs, resolved_files))
}

fn make_tree(
    _dirs: HashSet<PathBuf>,
    _files: HashSet<PathBuf>,
    _templates: HashSet<TemplateFile>,
) {
    todo!();
}

fn print_tree(
    dirs: HashSet<PathBuf>,
    files: HashSet<PathBuf>,
    templates: HashSet<TemplateFile>,
) {
    for dir in dirs {
        println!("  dir  -> {}", dir.as_os_str().to_str().unwrap());
    }

    for file in files {
        println!("  file -> {}", file.as_os_str().to_str().unwrap());
    }

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

pub fn make_project_tree(
    args: &SkelArgs,
    run_conf: &RunConfig,
) -> Result<(), Box<dyn Error>> {
    let templates = resolve_templates(run_conf)?;

    let mut dirs = resolve_dirs(run_conf)?;

    let (parent_dirs, files) = resolve_files(run_conf)?;

    for dir in parent_dirs.into_iter() {
        dirs.insert(dir);
    }

    if args.dry_run {
        print_tree(dirs, files, templates);
    } else {
        make_tree(dirs, files, templates);
    }

    Ok(())
}
