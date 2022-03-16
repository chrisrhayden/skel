// NOTE:
// 1) none of the functions do checks, Im not sure if or how that should change
// 2) the std lib said they might change create_dir_all or File::create
// make sure to adjust if they do

use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use crate::skeleton::Skeleton;

pub fn string_from_file<P>(path: P) -> Result<String, Box<dyn Error>>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    use std::io::{ErrorKind, Read};

    let mut include_file = match fs::File::open(&path) {
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Err(Box::from(format!(
                "file does not exists -- {:?}",
                path,
            )))
        }
        Err(err) => return Err(Box::from(err)),
        Ok(val) => val,
    };

    let mut buf = String::new();

    include_file.read_to_string(&mut buf)?;

    Ok(buf)
}

fn make_project_dirs(project: &Skeleton) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&project.project_root_string)?;

    if let Some(dir_iter) = project.dir_iter() {
        for dir in dir_iter {
            fs::create_dir_all(dir)?;
        }
    }

    Ok(())
}

fn make_project_files(project: &Skeleton) -> Result<(), Box<dyn Error>> {
    if let Some(file_iter) = project.file_iter() {
        for file in file_iter {
            fs::File::create(file)?;
        }
    }

    Ok(())
}

fn write_template(
    (template_path, template): (PathBuf, String),
) -> Result<(), io::Error> {
    use std::io::Write;

    let mut template_file = fs::File::create(template_path)?;

    if !template.is_empty() {
        template_file.write_all(template.as_bytes())?;
    }

    Ok(())
}

fn make_project_templates(project: &Skeleton) -> Result<(), Box<dyn Error>> {
    if let Some(template_iter) = project.template_iter() {
        for template in template_iter {
            write_template(template)?;
        }
    }

    Ok(())
}

/// the interface for making the project tree
/// this will make
///     - directory's, (mkdir -p path/to/dir)
///     - blank files (mkdir -p path/to && touch path/to/file)
///     - file templates (echo "$template" > path/to/file)
pub fn make_project_tree(project: &Skeleton) -> Result<(), Box<dyn Error>> {
    make_project_dirs(project)?;

    make_project_files(project)?;

    if project.dont_make_template {
        Ok(())
    } else {
        make_project_templates(project)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::path::PathBuf;

    use crate::test_utils::{make_fake_skeleton, TempSetup};

    #[test]
    fn test_make_project_dirs() {
        let mut temp = TempSetup::default();
        let project_root_path: PathBuf = temp.setup();

        let mut proj =
            make_fake_skeleton(Some(project_root_path.to_str().unwrap()));

        let mut src = project_root_path.clone();
        src.push("test_project");
        src.push("src");

        let test_dirs = proj.dirs.as_ref().unwrap().clone();

        // the make_project_dirs should not fail on making the same dir twice
        // add src again to test making twice
        proj.dirs.as_mut().unwrap().push(String::from("src"));

        if let Err(err) = make_project_dirs(&proj) {
            eprintln!("{}", err);

            panic!("make_project_dirs failed");
        }

        assert!(src.exists(), "didn't make the root src");

        for d in test_dirs {
            let mut dir_w_root = project_root_path.clone();

            dir_w_root.push(&proj.name);
            dir_w_root.push(&d);

            assert!(
                dir_w_root.exists(),
                "{:?} -- dir dose not exists",
                dir_w_root
            );
        }
    }

    #[test]
    fn test_make_project_files() {
        let mut temp = TempSetup::default();
        let project_root_path: PathBuf = temp.setup();

        temp.make_fake_skeleton_dirs(None)
            .expect("cant make temp dirs");

        let proj =
            make_fake_skeleton(Some(project_root_path.to_str().unwrap()));

        // dont bother testing make_project_dirs as that already being done and
        // if it fail then this function should fail
        if let Err(err) = make_project_dirs(&proj) {
            eprintln!("{}", err);

            panic!("make_project_dirs failed");
        }

        if let Err(err) = make_project_files(&proj) {
            eprintln!("{}", err);

            panic!("make_project_files failed");
        }

        let mut main_f = PathBuf::from(&proj.project_root_string);

        main_f.push("src");
        main_f.push("main.rs");

        assert!(main_f.exists(), "failed to make src/main.rs");

        for f in proj.file_iter().unwrap() {
            let mut file_w_root = project_root_path.clone();

            file_w_root.push(&proj.name);
            file_w_root.push(&f);

            assert!(
                file_w_root.exists(),
                "{:?} -- dir dose not exists",
                file_w_root
            );
        }
    }

    #[test]
    fn test_make_project_templates() {
        use std::io::Read;

        let mut temp = TempSetup::default();
        let root_buf = temp.setup();
        temp.make_fake_include().expect("cant make include file");

        temp.make_fake_skeleton_tree()
            .expect("cant make fake project");

        let proj = make_fake_skeleton(Some(root_buf.to_str().unwrap()));

        if let Err(err) = make_project_templates(&proj) {
            eprintln!("{}", err);
            panic!("didn't make templates");
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

        let test_main_file_string = "fn main() {\n    \
            println!(\"hello test_project\");\n}\n";

        assert_eq!(
            buf, test_main_file_string,
            "main_file template did not work"
        );

        let mut include_path = root_buf;
        include_path.push("test_include");

        let mut include_file =
            fs::File::open(include_path).expect("cant open include_file");

        let mut include_buf = String::new();

        include_file
            .read_to_string(&mut include_buf)
            .expect("cant read include_file");

        assert_eq!(
            include_buf, "test include value",
            "did not make include_file right"
        );
    }

    #[test]
    fn test_make_fake_project_tree() {
        use std::io::Read;

        let mut temp = TempSetup::default();
        let root = temp.setup();

        match make_project_tree(temp.skeleton.as_ref().unwrap()) {
            Ok(_) => {}
            Err(err) => panic!("Error: {}", err),
        };

        let mut project_root = root;
        project_root.push("test_project");

        let mut main_rs = project_root.clone();
        main_rs.push("src");
        main_rs.push("main.rs");

        assert!(main_rs.exists(), "main rs dose not exists");

        let mut main_file =
            fs::File::open(main_rs).expect("cant get main files");

        let mut buf = String::new();

        main_file
            .read_to_string(&mut buf)
            .expect("cant read file to string");

        let test_main_file_string = "fn main() {\n    \
            println!(\"hello test_project\");\n}\n";

        assert_eq!(
            buf, test_main_file_string,
            "main_file template did not work"
        );

        let mut more_test = project_root;
        more_test.push("tests/more_tests");
    }

    #[test]
    fn test_make_fake_project_tree_no_templating() {
        use std::io::Read;

        let mut temp = TempSetup::default();
        let root = temp.setup();

        temp.skeleton.as_mut().unwrap().templates.take();

        match make_project_tree(temp.skeleton.as_ref().unwrap()) {
            Ok(_) => {}
            Err(err) => panic!("Error: {}", err),
        };

        let mut project_root = root;
        project_root.push("test_project");

        let mut main_rs = project_root.clone();
        main_rs.push("src");
        main_rs.push("main.rs");

        assert!(main_rs.exists(), "main rs dose not exists");

        let mut main_file =
            fs::File::open(main_rs).expect("cant get main files");

        let mut buf = String::new();

        main_file
            .read_to_string(&mut buf)
            .expect("cant read file to string");

        let test_main_file_string = "";

        assert_eq!(
            buf, test_main_file_string,
            "main_file template did not work"
        );

        let mut more_test = project_root;
        more_test.push("tests/more_tests");
    }
}
