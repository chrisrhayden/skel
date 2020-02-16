use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::new_rs_error::{NewRsErrorType, NewRsError};
use crate::project::Project;

// the only function to do fs system checks is the make_project_tree,
// im not sure if that should change

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

fn make_template(template: (PathBuf, String)) -> Result<(), Box<dyn Error>> {
    use std::io::Write;

    let mut template_file = fs::File::create(template.0)?;

    template_file.write_all(template.1.as_bytes())?;

    Ok(())
}

fn make_project_templates(project: &Project) -> Result<(), Box<dyn Error>> {
    for template in project.template_iter().expect("no templates given") {
        make_template(template)?;
    }

    Ok(())
}

fn io_err_to_new_error(io_err: Box<dyn Error>) -> Result<(), NewRsError> {
    let err_string = format!("{:?}", io_err);

    let err_type = NewRsErrorType::IoError;

    Err(NewRsError::new(err_type, err_string))
}

// the interface for making the project tree
// this will make
//     - directory's,
//     - blank files
//     - file templates
// these functions works like unix's mkdir or touch
// if the std lib changes make sure to adjust
// templates are another story
pub fn make_project_tree(project: &Project) -> Result<(), NewRsError> {
    if project.root.exists() {
        let root_string = project.root_string();

        let err_type = NewRsErrorType::ProjectExists;

        return Err(NewRsError::new(err_type, root_string));
    }

    if let Err(err) = make_project_dirs(project) {
        io_err_to_new_error(err)?;
    }

    if let Err(err) = make_project_files(project) {
        io_err_to_new_error(err)?;
    }

    if let Err(err) = make_project_templates(project) {
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

    #[test]
    fn test_make_project_dirs() {
        let mut temp = TempSetup::default();
        let root_path: PathBuf = temp.setup();

        let mut proj = make_fake_project(Some(root_path.clone()));

        let mut src = root_path.clone();
        src.push("test_project");
        src.push("src");

        // the make_project_dirs should not fail on making the same dir twice
        // add src again to test making twice
        proj.dirs.push(String::from("src"));

        if let Err(err) = make_project_dirs(&proj) {
            eprintln!("{}", err);

            assert!(false, "make_project_dirs failed");
        }

        assert!(src.exists(), "didn't make the root src");

        for d in proj.dir_iter() {
            assert!(d.exists(), "{:?} -- dir dose not exists", d);
        }

        assert!(true);
    }

    #[test]
    fn test_make_project_files() {
        let mut temp = TempSetup::default();
        let root_path: PathBuf = temp.setup();
        temp.make_fake_project_dirs(None)
            .expect("cant make temp dirs");

        let proj = make_fake_project(Some(root_path));

        // dont bother testing make_project_dirs as that already being done and
        // if it fail then this function should fail
        if let Err(err) = make_project_dirs(&proj) {
            eprintln!("{}", err);

            assert!(false, "make_project_dirs failed");
        }

        if let Err(err) = make_project_files(&proj) {
            eprintln!("{}", err);

            assert!(false, "make_project_files failed");
        }

        let mut main_f = proj.root.clone();
        main_f.push("src");
        main_f.push("main.rs");

        assert!(main_f.exists(), "failed to make src/main.rs");

        for f in proj.file_iter() {
            assert!(f.exists(), "file dose not exists");
        }
    }

    // make_project_files should fail when called without the folder structure
    #[test]
    fn test_make_project_files_fails() {
        let proj = make_fake_project(None);

        if let Err(_) = make_project_files(&proj) {
            // TODO: test for specific failure maybe
            assert!(true);
        } else {
            assert!(false, "make_project_files worked?");
        }
    }

    #[test]
    fn test_make_project_root_exits() {
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

    #[test]
    fn test_make_project_templates() {
        use std::io::Read;

        let mut temp = TempSetup::default();
        let root_buf = temp.setup();

        temp.make_fake_project_tree()
            .expect("cant make fake project");

        let proj = make_fake_project(Some(root_buf.clone()));

        if let Err(err) = make_project_templates(&proj) {
            eprintln!("{}", err);
            assert!(false, "didn't make templates");
        };

        let mut main_rs = root_buf.clone();
        main_rs.push("test_project");
        main_rs.push("src");
        main_rs.push("main.rs");

        assert!(main_rs.exists(), "main rs dose not exists");

        let mut main_file =
            fs::File::open(main_rs).expect("cant get main files");

        let mut buf = String::new();

        main_file
            .read_to_string(&mut buf)
            .expect("cant read file to string");

        let test_main_file_string = r#"fn main() {
    println!("hello test_project");
}
"#;

        assert_eq!(
            buf, test_main_file_string,
            "main_file template did not work"
        );
    }
}
