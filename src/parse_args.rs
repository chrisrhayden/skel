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

// NOTE: this is a test function
pub fn make_args() -> SkelArgs {
    SkelArgs {
        alt_config_path: Some("./docs/example.config.toml".into()),
        skeleton: Some(String::from("py")),
        name: Some(String::from("fuck")),
        dry_run: true,
        ..Default::default()
    }
}

pub fn parse_args() -> Result<SkelArgs, Box<dyn Error>> {
    let mut skel_args = SkelArgs::parse();
    println!("{:#?}", skel_args);

    if skel_args.skeleton.is_none() && skel_args.skeleton_file.is_none() {
        return Err(Box::from(String::from(
            "did not get a skeleton or skeleton-file to make",
        )));
    }

    if skel_args.name.is_some()
        && skel_args.skeleton_file.is_some()
        && skel_args.skeleton.is_some()
    {
        return Err(Box::from(String::from("did not get correct args")));
    }

    if skel_args.name.is_none()
        && (skel_args.skeleton.is_some() && skel_args.skeleton_file.is_some())
    {
        skel_args.name = Some(skel_args.skeleton.as_ref().unwrap().to_owned());
    }

    Ok(skel_args)
}
