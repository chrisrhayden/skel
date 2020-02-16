use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use clap::{App, Arg};
use serde::Deserialize;

use new_rs::{collect_project_config, make_project, Project};

type NewResult<T> = Result<T, Box<dyn Error>>;
type UserResult = NewResult<UserConfig>;

// TODO: consider using the &str's form clap instead of new strings
#[derive(Default)]
struct NewArgs {
    pub root: Option<String>,
    pub name: Option<String>,
    pub type_user_config: bool,
    pub type_str: Option<String>,
    pub user_config_path: Option<String>,
}

fn parse_args() -> Result<NewArgs, Box<dyn Error>> {
    let matches = App::new("new, a project maker")
        .about("makes projects from ether a file or directory structure")
        .version("0")
        .arg(
            Arg::with_name("TYPE")
                .takes_value(true)
                .required_unless("project file")
                .help("a project type to make"),
        )
        .arg(
            Arg::with_name("NAME")
                .takes_value(true)
                .required(false)
                .help("the project to make"),
        )
        .arg(
            Arg::with_name("project file")
                .short('p')
                .long("project-file")
                .value_name("FILE")
                .takes_value(true)
                .help("a project file"),
        )
        .arg(
            Arg::with_name("different root")
                .short('r')
                .long("different-root")
                .value_name("PATH")
                .help("use PATH as root inserted of current dir"),
        )
        .arg(
            Arg::with_name("different config")
                .short('c')
                .long("different config")
                .value_name("FILE"),
        )
        .get_matches();

    let project_type = matches.value_of("TYPE");
    let name = matches.value_of("NAME");
    let project_file = matches.value_of("project file");
    let root = matches.value_of("different root");
    let config_path = matches.value_of("different config");

    // println!("name {:?}", name);
    // println!("root {:?}", root);
    // println!("project_type {:?}", project_type);
    // println!("project_file {:?}", project_file);
    // println!("config_path {:?}", config_path);

    if project_type.is_some() && project_file.is_some() && name.is_some() {
        return Err(Box::from(String::from("to many args given")));
    } else if name.is_none()
        && (project_type.is_none() && project_file.is_none())
    {
        return Err(Box::from(String::from("to few args given")));
    }

    let mut new_args = NewArgs::default();

    new_args.root = root.map(String::from);
    new_args.user_config_path = config_path.map(String::from);

    // TODO: make clap do this or find something else maybe
    if project_type.is_some() && project_file.is_none() {
        new_args.type_str = project_type.map(String::from);

        new_args.type_user_config = true;
    } else if project_type.is_some() && project_file.is_some() && name.is_none()
    {
        new_args.type_str = project_file.map(String::from);

        new_args.type_user_config = false;
    } else {
        return Err(Box::from(String::from("bad arg")));
    };

    if name.is_none() && project_type.is_some() {
        new_args.name = project_type.map(String::from);
    } else if name.is_some() {
        new_args.name = name.map(String::from);
    } else {
        return Err(Box::from(String::from("bad args or bad parsing of args")));
    };

    Ok(new_args)
}

fn get_home_dir() -> PathBuf {
    let home_path = env::var("HOME").expect("cant get env var HOME");

    PathBuf::from(home_path)
}

#[derive(Debug, Deserialize)]
struct UserConfig {
    projects: HashMap<String, String>,
    alias: HashMap<String, Vec<String>>,
}

fn config_string_default_or(
    config_path: &Option<String>,
) -> (String, Option<String>) {
    if let Some(config_path) = config_path {
        (config_path.to_owned(), None)
    } else {
        let mut config_dir = get_home_dir();
        config_dir.push(".config");
        config_dir.push("new");
        let mut config_path = config_dir.clone();
        config_path.push("new_config.toml");

        (
            config_path
                .to_str()
                .expect("cant make str from default config  file")
                .to_owned(),
            Some(
                config_dir
                    .to_str()
                    .expect("cant make str from default config  file")
                    .to_owned(),
            ),
        )
    }
}

fn root_string_default_or(root: &Option<String>) -> String {
    if let Some(root) = root {
        root.to_owned()
    } else {
        // default to current_dir
        env::current_dir()
            .expect("cant get current_dir")
            .to_str()
            .expect("cant convet current dir to string")
            .to_owned()
    }
}

fn get_user_config(config_str: &str) -> UserResult {
    use std::io::Read;

    let config_path = PathBuf::from(config_str);

    if !config_path.exists() {
        return Err(Box::from(format!(
            "config dose not exists -- {}",
            config_path.to_str().expect("cant get root path"),
        )));
    }

    // TODO: more gracefully hand this
    let mut conf_file = fs::File::open(&config_path)
        .expect(&format!("can open file config path {:?}", config_path));

    let mut config_str = String::new();

    conf_file
        .read_to_string(&mut config_str)
        .expect("cant read to string");

    // TODO: let the user know whats wrong in a nice way
    let toml_conf = toml::from_str::<UserConfig>(&config_str)
        .expect(&format!("TOML Error -- {}", config_str));

    Ok(toml_conf)
}

fn template_user_config(user_str: &str, old_string: &str) -> String {
    old_string.replace("{{new-config}}", user_str)
}

fn find_project_file(
    user_config: &UserConfig,
    type_str: &str,
) -> NewResult<String> {
    let type_string = type_str.to_string();

    if let Some(path_string) = user_config.projects.get(&type_string) {
        return Ok(path_string.clone());
    }

    let mut project_string = String::new();
    for (project, alias) in user_config.alias.iter() {
        // println!("--->>>>> {:?} {:?}", project, alias);
        if alias.contains(&type_string) && project_string.is_empty() {
            project_string.push_str(project);
        }
    }

    if project_string.is_empty() {
        panic!("nothing is in project_string");
    }

    match user_config.projects.get(&project_string) {
        Some(val) => Ok(val.to_string()),
        None => {
            // this seams unlikely
            Err(Box::from(format!(
                "no project for that ailas -- {}",
                type_string
            )))
        }
    }
}

fn project_path_with_templateing(
    args: &NewArgs,
    user_config: &UserResult,
    config_dir: &Option<String>,
) -> NewResult<PathBuf> {
    let project_pathbuf = if let Some(proj_str) = args.type_str.as_ref() {
        if args.type_user_config && user_config.is_err() {
            if let Err(err) = user_config {
                return Err(Box::from(format!("{}", err)));
            } else {
                unreachable!();
            }
        } else if args.type_user_config && user_config.is_ok() {
            let p_string =
                find_project_file(user_config.as_ref().unwrap(), &proj_str)?;

            let p_string = if let Some(c_dir) = config_dir {
                template_user_config(c_dir, &p_string)
            } else {
                panic!("did not get project_dir string");
            };

            PathBuf::from(p_string)
        } else {
            PathBuf::from(proj_str)
        }
    } else {
        // we should not reach this
        panic!("did not receive the project type or file");
    };

    Ok(project_pathbuf)
}

// last takes precedents:
//      default > config > cli config
fn resolve_default(
    args: &NewArgs,
    user_config: &UserResult,
    config_dir: &Option<String>,
) -> NewResult<Project> {
    // if we dont have a name here we never will
    let name = match args.name.clone() {
        None => return Err(Box::from(String::from("did not get name"))),
        Some(val) => val,
    };

    let project_pathbuf =
        project_path_with_templateing(&args, user_config, config_dir)?;

    // return if project path dose not exists, we will be able to make projects
    // from directory's eventually
    if !project_pathbuf.exists() {
        return Err(Box::from(format!(
            "project path given dos not exists -- {}",
            project_pathbuf.to_str().unwrap()
        )));
    }

    let project_config = collect_project_config(&project_pathbuf)?;

    let mut root_string = root_string_default_or(&args.root);
    // set root to the project name not the current_dir
    // or the one given on the cli
    // TODO: make this generic for windows maybe
    root_string.push('/');
    root_string.push_str(&name);

    Ok(Project::new(root_string, name, project_config))
}

fn main() {
    let args = match parse_args() {
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
        Ok(val) => val,
    };

    let (config_path, config_dir) =
        config_string_default_or(&args.user_config_path);

    let user_config = get_user_config(&config_path);

    let project = match resolve_default(&args, &user_config, &config_dir) {
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
        Ok(val) => val,
    };

    if let Err(err) = make_project(&project) {
        eprintln!("{}", err);
    }
}
