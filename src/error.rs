use std::io;

use miette::Diagnostic;
use thiserror::Error;

use crate::web_api::error::ApiError;

pub(crate) type LibResult<T> = Result<T>;
pub(crate) type LibError = Error;

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

    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
}
