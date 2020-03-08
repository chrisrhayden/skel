use std::{error::Error, path::PathBuf, process::Command};

use crate::project::Project;

fn make_bash_string(project: &Project) -> String {
    // if unwind fails here we have other issues
    let mut bash_string = project
        .build
        .as_ref()
        .expect("cant get build script")
        .to_string();

    bash_string.insert_str(0, "#!/usr/bin/env bash\n\n");

    project.template_str(&bash_string)
}

fn make_cmd(root: &PathBuf, bash_str: &str) -> Command {
    let mut cmd: Command = Command::new("bash");

    cmd.arg("-c");

    cmd.arg(bash_str);

    // only switch dirs if the root has been made
    if root.exists() {
        cmd.current_dir(root);
    }

    cmd
}

fn run_cmd(cmd: &mut Command) -> Result<(), Box<dyn Error>> {
    let output = match cmd.output() {
        Ok(val) => val,
        Err(err) => {
            return Err(Box::from(format!("Bad Command: {}", err)));
        }
    };

    if output.status.success() {
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    } else {
        Err(Box::from(format!(
            "Command Error {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub fn call_build_script(project: &Project) -> Result<(), Box<dyn Error>> {
    if project.build.is_none() {
        return Err(Box::from(String::from(
            "call_build_script was called without a build script to use",
        )));
    }

    let bash_string = make_bash_string(project);

    let mut cmd = make_cmd(&project.root_path, &bash_string);

    run_cmd(&mut cmd)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::make_fake_project;

    // TODO: test if the cmd was made correctly or if ran correctly, but idk how

    #[test]
    fn test_make_bash_string() {
        let fake_root = Some(PathBuf::from("/tmp/test_root"));
        let proj = make_fake_project(fake_root);

        let new_string = make_bash_string(&proj);
        let hand_made = String::from(
            r#"#!/usr/bin/env bash

if [[ -d test_project ]]; then
    echo "running in $PWD"
fi"#,
        );

        assert_eq!(new_string, hand_made, "didn't make string correctly");
    }
}
