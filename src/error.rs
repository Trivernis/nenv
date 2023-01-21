use std::io;

use miette::Diagnostic;
use thiserror::Error;

use crate::{
    mapper::error::MapperError,
    repository::{config::ConfigError, extract::ExtractError},
    web_api::error::ApiError,
};

pub(crate) type LibResult<T> = Result<T>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("Failed to call nodejs.com api: {0}")]
    Web(
        #[from]
        #[source]
        #[diagnostic_source]
        ApiError,
    ),
    #[error("Failed to extract archive: {0}")]
    Extract(
        #[from]
        #[source]
        #[diagnostic_source]
        ExtractError,
    ),

    #[error("Failed to load config file: {0}")]
    Config(
        #[from]
        #[source]
        #[diagnostic_source]
        ConfigError,
    ),
    #[error("Mapper failed: {0}")]
    Mapper(
        #[from]
        #[source]
        #[diagnostic_source]
        MapperError,
    ),
    #[error("Failed to work with json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
}
