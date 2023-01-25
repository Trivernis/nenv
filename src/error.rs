use std::{ffi::OsString, path::PathBuf};

use miette::{Diagnostic, NamedSource, SourceSpan};

use thiserror::Error;

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
        let mut src = src.to_string();
        let mut pos = (0, src.len()).into();
        let clean_src = src.trim_start_matches('^');

        if let Some((arg_str, arg_pos)) = find_in_args(&clean_src) {
            pos = arg_pos;
            src = arg_str;
        }

        Self {
            src,
            detail: detail.to_string(),
            pos,
        }
    }

    pub fn unknown_version<S: ToString>(src: S) -> Self {
        Self::new(src, "Unknown version.")
    }

    pub fn unfulfillable_version<S: ToString>(src: S) -> Self {
        Self::new(src, "The version requirement cannot be fulfilled.")
    }

    pub fn not_installed<S: ToString>(src: S) -> Self {
        Self::new(src, "The version is not installed.")
    }

    pub fn unsupported<S: ToString>(src: S) -> Self {
        Self::new(
            src,
            "This type of version string is not supported with this operation.",
        )
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse the contents as JSON.")]
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
#[diagnostic(code(nenv::bincode::serialize))]
#[error("failed to serialize value to bincode")]
pub struct SerializeBincodeError {
    #[from]
    caused_by: bincode::Error,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::toml::deserialize))]
#[error("The config file could not parsed.")]
pub struct ParseConfigError {
    #[source_code]
    src: NamedSource,

    #[label]
    pos: Option<SourceSpan>,

    #[source]
    caused_by: toml::de::Error,
}

impl ParseConfigError {
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
            pos: abs_pos,
            caused_by,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::toml::serialize))]
#[error("Failed to serialize the value to toml string.")]
pub struct SerializeTomlError {
    #[from]
    caused_by: toml::ser::Error,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(nenv::http))]
#[error("http request failed")]
pub struct ReqwestError(#[from] reqwest::Error);

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(
    code(nenv::exec::command),
    help("Make sure you selected the correct node version and check if {path:?} exists.")
)]
#[error("The command `{command}` could not be found for this node version.")]
pub struct CommandNotFoundError {
    command: String,

    #[source_code]
    full_command: String,

    path: PathBuf,

    #[label("this command")]
    pos: SourceSpan,
}

impl CommandNotFoundError {
    pub fn new(command: String, args: Vec<OsString>, path: PathBuf) -> Self {
        let (full_command, pos) = find_in_args(&command).unwrap_or_else(|| {
            let pos = (0, command.len()).into();
            let full_command = format!(
                "{command} {}",
                args.into_iter()
                    .map(|a| a.into_string().unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            (full_command, pos)
        });

        Self {
            command,
            full_command,
            path,
            pos,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to create mappings to directory {dir:?}.")]
#[diagnostic(
    code(nenv::map::command),
    help("Check if this node version was installed correctly.")
)]
pub struct MapDirError {
    pub dir: PathBuf,

    #[source]
    pub caused_by: std::io::Error,
}

pub fn find_in_args(query: &str) -> Option<(String, SourceSpan)> {
    let args_string = std::env::args().fold(String::new(), |s, acc| format!("{s} {acc}"));

    args_string
        .find(&query)
        .map(|index| (args_string, (index, query.len()).into()))
}
