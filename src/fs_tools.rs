use std::error::Error;
use std::fs;

use crate::new_rs_error::{NewRsErrorType, NewRsError};
use crate::project::Project;

fn make_project_dirs(project: &Project) -> Result<(), Box<dyn Error>> {
    for dir in project.dir_iter() {
        fs::create_dir_all(dir)?;
    }

    Ok(())
}

fn make_project_files(project: &Project) -> Result<(), Box<dyn Error>> {
    for file in project.file_iter() {
        fs::File::create(file)?;
    }

    Ok(())
}

fn io_err_to_new_error(io_err: Box<dyn Error>) -> Result<(), NewRsError> {
    let err_string = format!("{:?}", io_err);

    let err_type = NewRsErrorType::IoError;

    Err(NewRsError::new(err_type, err_string))
}

pub fn make_project_tree(project: &Project) -> Result<(), NewRsError> {
    if project.root.exists() {
        let root = project
            .root
            .as_os_str()
            .to_str()
            .expect("cant get project root");

        let err_type = NewRsErrorType::ProjectExists;

        return Err(NewRsError::new(err_type, root.to_string()));
    }

    if let Err(err) = make_project_dirs(project) {
        io_err_to_new_error(err)?;
    }

    if let Err(err) = make_project_files(project) {
        io_err_to_new_error(err)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::path::PathBuf;

    use crate::test_utils::{TempSetup, make_fake_project};

    // we would need to make the directory anyway so just test both making the
    // files and dirs at once
    #[test]
    fn test_make_project_tree() {
        let mut temp = TempSetup::default();
        let root_path: PathBuf = temp.setup();

        let mut proj = make_fake_project(Some(root_path.clone()));

        let mut src = root_path.clone();
        src.push("test_project");

        src.push("src");

        // should try and make this like the mkdir in unix
        proj.dirs.push(String::from("src"));

        if let Err(err) = make_project_dirs(&proj) {
            eprintln!("{}", err);

            assert!(false, "make_project_dirs failed");
        }

        assert!(src.exists(), "didn't make the root src");

        for d in proj.dir_iter() {
            assert!(d.exists(), "dir dose not exists");
        }

        assert!(true);

        if let Err(err) = make_project_files(&proj) {
            eprintln!("{}", err);

            assert!(false, "make_project_files failed");
        }

        let mut root = proj.root.clone();

        root.push("src");

        assert!(root.exists(), "failed to make root");

        for f in proj.file_iter() {
            assert!(f.exists(), "file dose not exists");
        }
    }

    #[test]
    fn test_root_exits() {
        let mut temp = TempSetup::default();

        let root_path: PathBuf = temp.setup();

        let proj = make_fake_project(Some(root_path.clone()));

        let mut src = root_path.clone();
        src.push("test_project");
        src.push("src");

        if let Err(err) = fs::create_dir_all(&src) {
            eprintln!("{}", err);
            assert!(false, "something is fucked");
        }

        if let Err(err) = make_project_tree(&proj) {
            match err.kind() {
                NewRsErrorType::ProjectExists => assert!(true),
                NewRsErrorType::IoError => {
                    eprintln!("{}", err);
                    assert!(false, "io err");
                }
            }
        } else {
            assert!(false, "did not fail");
        };

        assert!(true);
    }
}
