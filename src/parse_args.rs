use std::error::Error;

use clap::Parser;

/// make a project from a skeleton defined in a toml file
#[derive(Parser, Default, Debug)]
pub struct SkelArgs {
    /// the skeleton to make, can be the skeleton name or alias
    pub skeleton: Option<String>,
    /// the name of the new project to make
    pub name: Option<String>,
    #[clap(short, long)]
    /// a path to a skeleton file
    pub skeleton_file: Option<String>,
    #[clap(short, long)]
    /// a path to a main config file
    pub alt_config_path: Option<String>,
    #[clap(short = 'D', long)]
    /// a different root to make the project in to
    pub different_root: Option<String>,
    #[clap(short, long)]
    /// print out what will be done
    pub dry_run: bool,
}

// TODO: make a better error messages
// NOTE: i dont know a way to get clap to parse the args the way i want
pub fn parse_args() -> Result<SkelArgs, Box<dyn Error>> {
    let mut skel_args = SkelArgs::parse();

    if skel_args.skeleton.is_none() && skel_args.skeleton_file.is_none() {
        return Err(Box::from(String::from(
            "Error: did not get a skeleton or skeleton-file to make",
        )));
    }

    if skel_args.name.is_some()
        && skel_args.skeleton_file.is_some()
        && skel_args.skeleton.is_some()
    {
        return Err(Box::from(String::from(
            "Error: both a skeleton and a skeleton-file given",
        )));
    }

    if skel_args.skeleton.is_some()
        && (skel_args.name.is_none() && skel_args.skeleton_file.is_none())
    {
        return Err(Box::from(String::from("Error: did not get enough args")));
    }

    if skel_args.skeleton_file.is_some()
        && (skel_args.skeleton.is_none() && skel_args.name.is_none())
    {
        return Err(Box::from(String::from(
            "Error: did not get a project name to make",
        )));
    }

    // if `name` is none and `skeleton` and `skeleton_file` is some then the
    // `skeleton` variable is the `name`
    if skel_args.name.is_none()
        && (skel_args.skeleton.is_some() && skel_args.skeleton_file.is_some())
    {
        skel_args.name = skel_args.skeleton.take();
    }

    Ok(skel_args)
}
