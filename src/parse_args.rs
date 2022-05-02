use std::error::Error;

use clap::Parser;

#[derive(Parser, Default, Debug)]
pub struct SkelArgs {
    pub skeleton: Option<String>,
    pub name: Option<String>,
    #[clap(short, long)]
    pub skeleton_file: Option<String>,
    #[clap(short, long)]
    pub alt_config_path: Option<String>,
    #[clap(short = 'D', long)]
    pub different_root: Option<String>,
    #[clap(short, long)]
    pub dry_run: bool,
}

// TODO: make a better error message
// NOTE: i dont know a way to get clap to parse the args the way i want
pub fn parse_args() -> Result<SkelArgs, Box<dyn Error>> {
    let mut skel_args = SkelArgs::parse();

    if skel_args.skeleton.is_none() && skel_args.skeleton_file.is_none() {
        return Err(Box::from(String::from(
            "did not get a skeleton or skeleton-file to make",
        )));
    }

    if skel_args.name.is_some()
        && skel_args.skeleton_file.is_some()
        && skel_args.skeleton.is_some()
    {
        return Err(Box::from(String::from(
            "Error: both a skeleton file and a skeleton project given",
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
    // `skeleton` variable is where `name` is
    if skel_args.name.is_none()
        && (skel_args.skeleton.is_some() && skel_args.skeleton_file.is_some())
    {
        skel_args.name = skel_args.skeleton.take();
    }

    Ok(skel_args)
}
