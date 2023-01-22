use miette::{Diagnostic, NamedSource, SourceSpan};

use thiserror::Error;

use crate::{mapper::error::MapperError, repository::extract::ExtractError};

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[diagnostic(code(nenv::extract))]
    #[error("The node archive could not be extracted")]
    Extract(#[from] ExtractError),

    #[diagnostic(code(nenv::mapper))]
    #[error("Mapping failed")]
    Mapper(#[from] MapperError),

    #[diagnostic(code(nenv::version))]
    #[error("The passed version is invalid")]
    Version(#[from] VersionError),
}

#[derive(Debug, Error, Diagnostic)]
#[error("{detail}")]
#[diagnostic(code(nenv::version), help("Make sure there's no typo in the version."))]
pub struct VersionError {
    #[source_code]
    src: String,

    #[label("this version")]
    pos: SourceSpan,

    detail: String,
}

impl VersionError {
    pub fn new<S1: ToString, S2: ToString>(src: S1, detail: S2) -> Self {
        let src = src.to_string();
        let pos = (0, src.len()).into();

        Self {
            src,
            detail: detail.to_string(),
            pos,
        }
    }

    pub fn unknown_version<S: ToString>(src: S) -> Self {
        Self::new(src, "unknown version")
    }

    pub fn unfulfillable_version<S: ToString>(src: S) -> Self {
        Self::new(src, "the version requirement cannot be fulfilled")
    }

    pub fn not_installed<S: ToString>(src: S) -> Self {
        Self::new(src, "the version is not installed")
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("failed to parse json")]
#[diagnostic(code(nenv::json::deserialize))]
pub struct ParseJsonError {
    #[source_code]
    pub src: NamedSource,

    #[label]
    pub pos: SourceSpan,

    #[source]
    pub caused_by: serde_json::Error,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::json::serialize))]
#[error("failed to serialize value to json string")]
pub struct SerializeJsonError {
    #[from]
    caused_by: serde_json::Error,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::toml::deserialize))]
#[error("failed to parse toml value")]
pub struct ParseTomlError {
    #[source_code]
    src: NamedSource,

    #[label]
    pos: Option<SourceSpan>,

    #[source]
    caused_by: toml::de::Error,
}

impl ParseTomlError {
    pub fn new(file_name: &str, src: String, caused_by: toml::de::Error) -> Self {
        let abs_pos = caused_by
            .line_col()
            .map(|(l, c)| {
                src.lines()
                    .into_iter()
                    .take(l)
                    .map(|line| line.len() + 1)
                    .sum::<usize>()
                    + c
            })
            .map(|p| SourceSpan::new(p.into(), 0.into()));
        Self {
            src: NamedSource::new(file_name, src),
            pos: abs_pos.into(),
            caused_by,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::toml::serialize))]
#[error("failed to serialize value to toml string")]
pub struct SerializeTomlError {
    #[from]
    caused_by: toml::ser::Error,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::http))]
#[error("http request failed")]
pub struct ReqwestError(#[from] reqwest::Error);
