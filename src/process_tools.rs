use std::{
    error::Error,
    path::Path,
    process::{Command, Stdio},
};

use crate::skeleton::Skeleton;

fn make_bash_string(project: &Skeleton) -> String {
    // project.build should always exist because this is only run if it dose
    let mut bash_string = project
        .build
        .as_ref()
        .expect("cant unwrap build script")
        .to_string();

    bash_string.insert_str(0, "#!/usr/bin/env bash\n\n");

    project.run_template(&bash_string)
}

fn make_cmd<P: AsRef<Path>>(
    root: P,
    bash_str: &str,
    show_output: bool,
) -> Command {
    let mut cmd = Command::new("bash");

    cmd.arg("-c").arg(bash_str);

    // only switch dirs if the root has been made
    // the build script will need to make the directory itself
    if root.as_ref().exists() {
        cmd.current_dir(root);
    }

    if !show_output {
        cmd.stdout(Stdio::null());
    }

    cmd
}

fn run_cmd(cmd: &mut Command) -> Result<(), Box<dyn Error>> {
    let mut running_cmd = match cmd.spawn() {
        Ok(val) => val,
        Err(err) => {
            return Err(Box::from(format!("Bad Command: {}", err)));
        }
    };

    match running_cmd.wait() {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::from(format!("Command Error: {}", err))),
    }
}

pub fn call_build_script(project: &Skeleton) -> Result<(), Box<dyn Error>> {
    // if no build script present then just return
    if project.build.is_none() {
        return Ok(());
    }

    let bash_string = make_bash_string(project);

    let mut cmd = make_cmd(
        &project.project_root_string,
        &bash_string,
        project.show_build_output,
    );

    run_cmd(&mut cmd)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::make_fake_skeleton;

    // TODO: test if the cmd was made correctly or if ran correctly, but idk how

    #[test]
    fn test_make_bash_string() {
        let fake_root = Some("/tmp/test_root");
        let proj = make_fake_skeleton(fake_root);

        let new_string = make_bash_string(&proj);
        let hand_made = String::from(
            "#!/usr/bin/env bash\n\n\
            touch test_build\n\
            if [[ -d test_project ]]; then\n    \
                echo \"running in $PWD\"\n\
            fi",
        );

        assert_eq!(new_string, hand_made, "didn't make string correctly");
    }
}
