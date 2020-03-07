use std::{error::Error, fmt};

// basic error type enum to pattern match on
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewInnerErrType {
    ProjectExists,
    IoError,
}

impl fmt::Display for NewInnerErrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            NewInnerErrType::ProjectExists => String::from("ProjectExists"),
            NewInnerErrType::IoError => String::from("IoError"),
        };

        write!(f, "{}", message)
    }
}

// a basic error for new_rs
// TODO: make this use an error trait
#[derive(Debug)]
pub struct NewInnerError {
    err_str: String,
    err_type: NewInnerErrType,
}

impl NewInnerError {
    pub fn new(err_type: NewInnerErrType, err_str: String) -> Self {
        Self { err_type, err_str }
    }

    pub fn io_error(err_string: &str) -> Self {
        Self {
            err_type: NewInnerErrType::IoError,
            err_str: err_string.to_owned(),
        }
    }

    pub fn from_io_err(io_err: Box<dyn Error>) -> NewInnerError {
        let err_string = format!("{:?}", io_err);

        let err_type = NewInnerErrType::IoError;

        NewInnerError::new(err_type, err_string)
    }

    // this is only used in test at the moment
    #[allow(dead_code)]
    pub fn kind(&self) -> NewInnerErrType {
        self.err_type
    }
}

impl fmt::Display for NewInnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error -- type: {} message: {}",
            self.err_type, self.err_str
        )
    }
}
