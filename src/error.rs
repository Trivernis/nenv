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
    #[error("Failed to call nodejs.com api.")]
    Web(
        #[from]
        #[source]
        #[diagnostic_source]
        ApiError,
    ),

    #[error("The node archive could not be extracted")]
    Extract(
        #[from]
        #[source]
        #[diagnostic_source]
        ExtractError,
    ),

    #[error("The config file could not be loaded")]
    Config(
        #[from]
        #[source]
        #[diagnostic_source]
        ConfigError,
    ),

    #[error("Mapping failed")]
    Mapper(
        #[from]
        #[source]
        #[diagnostic_source]
        MapperError,
    ),

    #[error("The passed is invalid")]
    Version(
        #[from]
        #[diagnostic_source]
        VersionError,
    ),

    #[error("Failed to work with json")]
    Json(#[from] serde_json::Error),

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
