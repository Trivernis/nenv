use std::io;

use miette::{Diagnostic, NamedSource, SourceSpan};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

use crate::consts::{CFG_DIR, CFG_FILE_PATH, NODE_DIST_URL};

use super::NodeVersion;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub dist_base_url: String,
    #[serde(with = "NodeVersion")]
    pub default_version: NodeVersion,
}

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Error, Diagnostic, Debug)]
pub enum ConfigError {
    #[diagnostic(code(nenv::config::io))]
    #[error("IO Error: {0}")]
    Io(
        #[from]
        #[source]
        io::Error,
    ),
    #[diagnostic(code(nenv::config::parse))]
    #[error("Failed to parse config file")]
    #[diagnostic_source]
    Parse {
        #[source_code]
        src: NamedSource,

        #[label]
        pos: Option<(usize, usize)>,

        #[source]
        e: toml::de::Error,
    },
    #[diagnostic(code(nenv::config::write))]
    #[error("Failed to serialize config file: {0}")]
    Serialize(
        #[from]
        #[source]
        toml::ser::Error,
    ),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dist_base_url: String::from(NODE_DIST_URL),
            default_version: NodeVersion::LatestLts,
        }
    }
}

impl Config {
    /// Loads the config file from the default config path
    pub async fn load() -> ConfigResult<Self> {
        if !CFG_FILE_PATH.exists() {
            if !CFG_DIR.exists() {
                fs::create_dir_all(&*CFG_DIR).await?;
            }
            let cfg = Config::default();
            cfg.save().await?;

            Ok(cfg)
        } else {
            let cfg_string = fs::read_to_string(&*CFG_FILE_PATH).await?;
            let cfg = toml::from_str(&cfg_string).map_err(|e| ConfigError::Parse {
                src: NamedSource::new("config.toml", cfg_string),
                pos: e.line_col(),
                e,
            })?;

            Ok(cfg)
        }
    }

    pub async fn save(&self) -> ConfigResult<()> {
        fs::write(&*CFG_FILE_PATH, toml::to_string_pretty(&self)?).await?;

        Ok(())
    }

    pub async fn set_default_version(&mut self, default_version: NodeVersion) -> ConfigResult<()> {
        self.default_version = default_version;
        self.save().await
    }
}
