use std::io;

use miette::Diagnostic;
use thiserror::Error;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Error, Diagnostic)]
pub enum ApiError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("{0}")]
    Other(#[help] String),
}

impl ApiError {
    pub fn other<S: ToString>(error: S) -> Self {
        Self::Other(error.to_string())
    }
}
