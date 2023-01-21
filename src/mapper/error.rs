use miette::Diagnostic;
use thiserror::Error;

use crate::repository::config::ConfigError;

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
}
