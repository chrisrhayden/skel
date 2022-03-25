#[derive(Default)]
pub struct SkelArgs {
    pub name: String,
    pub main_config_path: Option<String>,
    pub skeleton: Option<String>,
    pub skeleton_file: Option<String>,
    pub different_root: Option<String>,
    pub dry_run: Option<bool>,
}

// NOTE: this is a test function
pub fn make_args() -> SkelArgs {
    SkelArgs {
        main_config_path: None,
        skeleton: Some(String::from("py")),
        name: String::from("fuck"),
        dry_run: Some(true),
        ..Default::default()
    }
}
