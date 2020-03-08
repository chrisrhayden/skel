use std::{error::Error, fmt};

// basic error type enum to pattern match on
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkelErrType {
    ProjectExists,
    IoError,
}

impl fmt::Display for SkelErrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            SkelErrType::ProjectExists => String::from("ProjectExists"),
            SkelErrType::IoError => String::from("IoError"),
        };

        write!(f, "{}", message)
    }
}

// a basic error for new_rs
// TODO: make this use an error trait
#[derive(Debug)]
pub struct SkelError {
    err_str: String,
    err_type: SkelErrType,
}

impl SkelError {
    pub fn new(err_type: SkelErrType, err_str: String) -> Self {
        Self { err_type, err_str }
    }

    pub fn from_io_err(io_err: Box<dyn Error>) -> SkelError {
        let err_string = format!("{:?}", io_err);

        let err_type = SkelErrType::IoError;

        SkelError::new(err_type, err_string)
    }

    // this is only used in test at the moment
    #[allow(dead_code)]
    pub fn kind(&self) -> SkelErrType {
        self.err_type
    }
}

impl fmt::Display for SkelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error -- type: {} message: {}",
            self.err_type, self.err_str
        )
    }
}
