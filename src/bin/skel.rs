///! just a wrapper around the lib interface and the cli interface
use std::error::Error;

use skel::{
    cli::{parse_args, resolve_default},
    make_project,
};

// holy shit this is nice, though the errors might not be good for users
fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;

    let mut project = resolve_default(args)?;

    project.resolve_templates()?;

    make_project(&project)
}
