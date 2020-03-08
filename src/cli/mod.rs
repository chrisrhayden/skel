//! the cli interface for new
mod defaults;
mod parse_args;

pub use parse_args::{parse_args, NewArgs};

pub use defaults::{resolve_default, UserConfig};
