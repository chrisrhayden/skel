use std::error::Error;

use clap::{App, Arg, ArgMatches};

// TODO: consider using the &str's form clap instead of new strings
#[derive(Default, Debug)]
pub struct SkelArgs {
    pub name: String,
    pub type_str: String,
    pub different_root: Option<String>,
    pub cli_config_path: Option<String>,
    pub cli_project_file: Option<String>,
    pub dont_run_build: bool,
    pub dont_make_templates: bool,
    pub build_first: bool,
    pub show_build_output: bool,
}

impl SkelArgs {
    pub fn make_fake(name: &str, type_str: &str) -> Self {
        Self {
            name: name.to_owned(),
            type_str: type_str.to_owned(),
            different_root: None,
            cli_config_path: None,
            cli_project_file: None,
            dont_run_build: false,
            dont_make_templates: false,
            build_first: false,
            show_build_output: false,
        }
    }
}

fn get_arg_matches() -> ArgMatches {
    App::new("skel -- a project maker")
        .about("make a project from a toml file")
        .arg(
            Arg::with_name("project file")
                .short('p')
                .long("project-file")
                .value_name("FILE")
                .takes_value(true)
                .help("a project file to use instead of looking one up"),
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
                .long("different-config")
                .value_name("FILE")
                .help("use FILE instead of the default config file"),
        )
        .arg(
            Arg::with_name("no build")
                .short('n')
                .long("no-build")
                .takes_value(false)
                .help("dont run the build script"),
        )
        .arg(
            Arg::with_name("no templating")
                .short('N')
                .long("no-templating")
                .help("dont make templates"),
        )
        .arg(
            Arg::with_name("run build first")
                .short('b')
                .long("build-first")
                .help("run the build script before making the rest of the project"),
        )
        .arg(
            Arg::with_name("show build output")
                .short('o')
                .long("show-build-output")
                .help("show the output from the build script"),
        )
        .arg(
            Arg::with_name("TYPE")
                .takes_value(true)
                .help("a project type or alias to make"),
        )
        .arg(
            Arg::with_name("NAME")
                .takes_value(true)
                .required(false)
                .help("the name of the project to make"),
        )
        .get_matches()
}

pub fn parse_args() -> Result<SkelArgs, Box<dyn Error>> {
    let matches = get_arg_matches();

    let project_type = matches.value_of("TYPE");
    let name = matches.value_of("NAME");
    let project_file = matches.value_of("project file");
    let root = matches.value_of("different root");
    let config_path = matches.value_of("different config");

    // TODO: make clap actually do the semantics of this
    //       or maybe use another lib
    if project_type.is_some() && project_file.is_some() && name.is_some() {
        return Err(Box::from(String::from("to many args given")));
    } else if name.is_none()
        && (project_type.is_none() && project_file.is_none())
    {
        return Err(Box::from(String::from("to few args given")));
    }

    let mut skel_args = SkelArgs::default();

    if project_type.is_some() && project_file.is_none() {
        skel_args.type_str = project_type
            .map(String::from)
            .expect("cant unwrap project type");
    } else if project_type.is_none() && project_file.is_none() {
        return Err(Box::from(String::from("bad args")));
    };

    // i some how cant make clap exclusive for -p project_file and type_str so
    // if project_type is present but name is not
    // then project_type should be name
    if name.is_none() && project_type.is_some() {
        skel_args.name = project_type
            .map(String::from)
            .expect("cant unwrap project type");
    } else if name.is_some() {
        skel_args.name =
            name.map(String::from).expect("cant unwrap project name");
    } else {
        return Err(Box::from(String::from("bad args or bad parsing of args")));
    };

    skel_args.different_root = root.map(String::from);
    skel_args.cli_project_file = project_file.map(String::from);
    skel_args.cli_config_path = config_path.map(String::from);

    skel_args.dont_make_templates = matches.is_present("no templating");
    skel_args.dont_run_build = matches.is_present("no build");
    skel_args.build_first = matches.is_present("run build first");
    skel_args.show_build_output = matches.is_present("show build output");

    Ok(skel_args)
}
