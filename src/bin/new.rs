use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use clap::{App, Arg};
use serde::Deserialize;

use new_rs::{collect_config, make_project, Project};

// TODO: consider using the &str's form clap instead of new strings
#[derive(Default)]
struct NewArgs {
    pub root: Option<String>,
    pub name: Option<String>,
    pub type_user_config: bool,
    pub type_str: Option<String>,
}

fn parse_args() -> NewArgs {
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
        .get_matches();

    let project_type = matches.value_of("TYPE");
    let name = matches.value_of("NAME");
    let project_file = matches.value_of("project file");
    let root = matches.value_of("different root");

    if project_type.is_some() && project_file.is_some() && name.is_some() {
        panic!("to many args given");
    }

    let mut new_args = NewArgs::default();

    // TODO: make clap do this or find something else maybe
    if project_type.is_some() && project_file.is_none() {
        new_args.type_str = project_type.map(String::from);

        new_args.type_user_config = true;
    } else if project_type.is_some() && project_file.is_some() && name.is_none()
    {
        new_args.type_str = project_file.map(String::from);

        new_args.type_user_config = false;
    } else {
        panic!("bad arg");
    };

    if name.is_none() && project_type.is_some() {
        new_args.name = project_type.map(String::from);
    } else if name.is_some() {
        new_args.name = name.map(String::from);
    } else {
        panic!("bad args or bad parsing of args");
    };

    new_args.root = root.map(String::from);

    new_args
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

fn get_user_config() -> Option<UserConfig> {
    use std::io::Read;

    let mut config_path = get_home_dir();

    config_path.push(".config");
    config_path.push("new");
    config_path.push("new_config.toml");

    // if no config return silently and well take care of it when it matters
    if !config_path.exists() {
        return None;
    }

    let mut conf_file = match fs::File::open(&config_path) {
        Err(err) => panic!("Os Error -- {:?}", err),
        Ok(val) => val,
    };

    let mut config_str = String::new();

    conf_file
        .read_to_string(&mut config_str)
        .expect("cant read to string");

    // TODO: let the user know whats wrong ion a nice way
    let toml_conf = match toml::from_str::<UserConfig>(&config_str) {
        Err(err) => panic!("{}", err),
        Ok(val) => val,
    };

    Some(toml_conf)
}

fn find_project_file(user_config: &UserConfig, type_str: &str) -> PathBuf {
    let _project_path: String =
        if let Some(type_str) = user_config.projects.get(type_str) {
            type_str.to_owned()
        } else {
            let project_string = String::new();

            for key in user_config.alias.values() {
                println!("{:?}", key);
            }

            if project_string.is_empty() {
                panic!("nothing is in project_string");
            }

            project_string
        };

    unimplemented!();
}

// last takes precedents:
//      default > config > cli config
fn resolve_default(
    args: &NewArgs,
    user_config: Option<&UserConfig>,
) -> Result<Project, Box<dyn Error>> {
    let mut root: String = if let Some(root) = args.root.as_ref() {
        root.to_string()
    } else {
        // default to current_dir
        env::current_dir()
            .expect("cant get current_dir")
            .to_str()
            .expect("cant get string from current path")
            .to_string()
    };

    let name = args.name.clone().expect("did not get name");

    // set root to the project name not the current_dir
    // or the one given on the cli
    root.push('/');
    root.push_str(&name);

    let project_pathbuf = if let Some(proj_str) = args.type_str.as_ref() {
        if args.type_user_config && user_config.is_some() {
            find_project_file(user_config.unwrap(), &proj_str)
        } else if args.type_user_config && user_config.is_none() {
            return Err(Box::from(String::from(
                "no user config at ~/.config/new/new_config.toml",
            )));
        } else {
            PathBuf::from(proj_str)
        }
    } else {
        panic!("did not receive the project type or file");
    };

    if !project_pathbuf.exists() {
        return Err(Box::from(format!(
            "project file given dons not exists -- {}",
            project_pathbuf.to_str().unwrap()
        )));
    }

    let project_config = collect_config(&project_pathbuf)?;

    Ok(Project::new(root, name, project_config))
}

fn main() {
    let args = parse_args();

    let user_config = get_user_config();

    let project = match resolve_default(&args, user_config.as_ref()) {
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
