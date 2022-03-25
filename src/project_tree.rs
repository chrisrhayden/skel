use std::{collections::HashSet, error::Error, fs, path::PathBuf};

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
    template: &SkelTemplate,
    run_conf: &RunConfig,
) -> Result<TemplateFile, Box<dyn Error>> {
    let template_string = if let Some(include) = template.include.as_ref() {
        let include_path_string = run_conf
            .handle
            .render_template(include, &run_conf.template_data)?;

        let template_file_string = fs::read_to_string(&include_path_string)?;

        run_conf
            .handle
            .render_template(&template_file_string, &run_conf.template_data)?
    } else if let Some(template_file) = &template.template {
        run_conf
            .handle
            .render_template(template_file, &run_conf.template_data)?
    } else {
        return Err(Box::from(String::from(
            "no template string or include path for template",
        )));
    };

    let path = run_conf
        .handle
        .render_template(&template.path, &run_conf.template_data)?;

    let new_template = TemplateFile {
        path: path.into(),
        template: template_string,
    };

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
            let templated_dir = run_conf
                .handle
                .render_template(dir, &run_conf.template_data)?;

            resolved_dirs.insert(templated_dir.into());
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
            let file_string = run_conf
                .handle
                .render_template(file, &run_conf.template_data)?;

            let file_path = PathBuf::from(file_string);

            let parent = match file_path.parent() {
                None => {
                    return Err(Box::from(String::from(
                        "cant make project in root",
                    )))
                }
                Some(parent) => parent.to_owned(),
            };

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
        println!("  dir -> {}", dir.as_os_str().to_str().unwrap());
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
        // println!("----\n{}\n----", template.template);
    }
}

pub fn make_project_tree(
    args: &SkelArgs,
    run_conf: &RunConfig,
) -> Result<(), Box<dyn Error>> {
    let mut root_path = PathBuf::from(&run_conf.root_string);

    root_path.push(&args.name);

    if root_path.exists() {
        return Err(Box::from(format!(
            "root path exists {}",
            root_path.as_os_str().to_str().unwrap()
        )));
    }

    let templates = resolve_templates(run_conf)?;
    let mut dirs = resolve_dirs(run_conf)?;
    let (parent_dirs, files) = resolve_files(run_conf)?;

    for dir in parent_dirs.into_iter() {
        dirs.insert(dir);
    }

    dirs.insert(root_path);

    if args.dry_run.is_some() && args.dry_run.unwrap() {
        print_tree(dirs, files, templates);
    } else {
        make_tree(dirs, files, templates);
    }

    Ok(())
}
