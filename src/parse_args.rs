use std::path::PathBuf;

pub struct SkelArgs {
    pub main_config_path: Option<PathBuf>,
    pub skeleton: String,
}

// NOTE: this is a test function
pub fn make_args() -> SkelArgs {
    SkelArgs {
        main_config_path: None,
        skeleton: String::from("rs"),
    }
}
