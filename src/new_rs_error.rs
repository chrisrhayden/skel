use std::fmt;

// basic error type enum to pattern match on
#[derive(Debug, Clone, PartialEq)]
pub enum NewRsErrorType {
    ProjectExists,
    IoError,
}

impl fmt::Display for NewRsErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            NewRsErrorType::ProjectExists => String::from("ProjectExists"),
            NewRsErrorType::IoError => String::from("IoError"),
        };

        write!(f, "{}", message)
    }
}

// a basic error for new_rs
// TODO: make this use an error trait
#[derive(Debug)]
pub struct NewRsError {
    err_str: String,
    err_type: NewRsErrorType,
}

impl NewRsError {
    pub fn new(err_type: NewRsErrorType, err_str: String) -> Self {
        Self { err_type, err_str }
    }

    #[allow(dead_code)]
    pub fn kind(&self) -> NewRsErrorType {
        self.err_type.clone()
    }
}

impl fmt::Display for NewRsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type: {} message: {}", self.err_type, self.err_str)
    }
}
