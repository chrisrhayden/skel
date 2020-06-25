use std::error::Error;

use clap::{App, Arg, ArgMatches};

// TODO: consider using the &str's form clap instead of new strings
#[derive(Default, Debug)]
pub struct SkelArgs {
    pub name: String,
    pub alias_str: String,
    pub different_root: Option<String>,
    pub cli_config_path: Option<String>,
    pub cli_project_file: Option<String>,
    pub dont_run_build: bool,
    pub dont_make_templates: bool,
    pub build_first: bool,
    pub show_build_output: bool,
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
             Arg::with_name("ALIAS")
                 .takes_value(true)
                 .help("a project alias or alias to make"),
         )
         .arg(
             Arg::with_name("NAME")
                 .takes_value(true)
                 .required(false)
                 .help("the name of the project to make"),
         )
         .get_matches()
}

// check is the right amount of args is given
// TODO: make clap actually do the semantics of this
//       or maybe use another lib
fn project_info_check(
    project_alias: Option<&str>,
    project_file: Option<&str>,
    project_name: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    // if they all are present
    if project_alias.is_some()
        && project_file.is_some()
        && project_name.is_some()
    {
        Err(Box::from("can not use --project-file with an ALIAS"))
    // if they all are none
    } else if project_name.is_none()
        && project_alias.is_none()
        && project_file.is_none()
    {
        Err(Box::from("to few args given"))
    // if only an alias is given
    } else if project_alias.is_some()
        && (project_file.is_none() && project_name.is_none())
    {
        Err(Box::from("to few args given"))
    } else {
        Ok(())
    }
}

pub fn parse_args() -> Result<SkelArgs, Box<dyn Error>> {
    let matches = get_arg_matches();

    let project_alias = matches.value_of("ALIAS");
    let project_file = matches.value_of("project file");
    let project_name = matches.value_of("NAME");

    project_info_check(project_alias, project_file, project_name)?;

    let mut skel_args = SkelArgs::default();

    if project_alias.is_some() && project_file.is_none() {
        skel_args.alias_str = project_alias
            .map(String::from)
            .expect("cant unwrap project alias")
    };

    // project_info_check should handle all cases that are an error so just keep
    // it simple here
    if project_name.is_some() {
        skel_args.name = project_name
            .map(String::from)
            .expect("cant unwrap project name");
    } else if project_name.is_none() && project_alias.is_some() {
        skel_args.name = project_alias
            .map(String::from)
            .expect("cant unwrap project alias");
    }

    let different_root = matches.value_of("different root");
    let different_config_path = matches.value_of("different config");

    skel_args.different_root = different_root.map(String::from);
    skel_args.cli_project_file = project_file.map(String::from);
    skel_args.cli_config_path = different_config_path.map(String::from);

    skel_args.dont_make_templates = matches.is_present("no templating");
    skel_args.dont_run_build = matches.is_present("no build");
    skel_args.build_first = matches.is_present("run build first");
    skel_args.show_build_output = matches.is_present("show build output");

    Ok(skel_args)
}
