use std::{io, path::PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::repository::config::ConfigError;

use super::mapped_command::CommandError;

pub type MapperResult<T> = Result<T, MapperError>;

#[derive(Error, Diagnostic, Debug)]
pub enum MapperError {
    #[error("Config error: {0}")]
    Config(
        #[from]
        #[source]
        #[diagnostic_source]
        ConfigError,
    ),

    #[error("Failed to execute mapped command: {0}")]
    Command(#[from] CommandError),

    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to map directory {src:?}")]
    DirMapping {
        src: PathBuf,

        #[source]
        err: io::Error,
    },
}
