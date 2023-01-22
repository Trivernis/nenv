use std::io;

use miette::Diagnostic;
use semver::VersionReq;
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
    #[diagnostic(code(nenv::web))]
    #[error("Failed to call nodejs.com api.")]
    Web(
        #[from]
        #[source]
        #[diagnostic_source]
        ApiError,
    ),

    #[diagnostic(code(nenv::extract))]
    #[error("The node archive could not be extracted")]
    Extract(
        #[from]
        #[source]
        #[diagnostic_source]
        ExtractError,
    ),

    #[diagnostic(code(nenv::config))]
    #[error("The config file could not be loaded")]
    Config(
        #[from]
        #[diagnostic_source]
        ConfigError,
    ),

    #[diagnostic(code(nenv::mapper))]
    #[error("Mapping failed")]
    Mapper(
        #[from]
        #[source]
        #[diagnostic_source]
        MapperError,
    ),

    #[diagnostic(code(nenv::version))]
    #[error("The passed version is invalid")]
    Version(
        #[from]
        #[diagnostic_source]
        VersionError,
    ),

    #[diagnostic(code(nenv::json))]
    #[error("Failed to work with json")]
    Json(#[from] serde_json::Error),

    #[diagnostic(code(nenv::io))]
    #[error("Error during IO operation")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error, Diagnostic)]
pub enum VersionError {
    #[error("Invalid version string `{0}`")]
    ParseVersion(#[source_code] String),

    #[error("Unknown Version `{0}`")]
    UnkownVersion(#[source_code] String),

    #[error("The version `{0}` is not installed")]
    NotInstalled(#[source_code] String),

    #[error("The version requirement `{0}` cannot be fulfilled")]
    Unfulfillable(VersionReq),
}
