use miette::Context;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::SerializeTomlError;
use crate::{
    consts::{CFG_DIR, CFG_FILE_PATH, NODE_DIST_URL},
    error::ParseConfigError,
};

use super::NodeVersion;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Node execution related config
    pub node: NodeConfig,

    /// Configuration for how to download node versions
    pub download: DownloadConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeConfig {
    /// The default version if no version is specified
    /// in the `package.json` file or `NODE_VERSION` environment variable
    #[serde(with = "NodeVersion")]
    pub default_version: NodeVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadConfig {
    pub dist_base_url: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            default_version: NodeVersion::LatestLts,
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            dist_base_url: String::from(NODE_DIST_URL),
        }
    }
}

impl Config {
    /// Loads the config file from the default config path
    pub async fn load() -> Result<Self> {
        if !CFG_FILE_PATH.exists() {
            if !CFG_DIR.exists() {
                fs::create_dir_all(&*CFG_DIR)
                    .await
                    .into_diagnostic()
                    .context("creating config dir")?;
            }
            let cfg = Config::default();
            cfg.save().await?;

            Ok(cfg)
        } else {
            let cfg_string = fs::read_to_string(&*CFG_FILE_PATH)
                .await
                .into_diagnostic()
                .context("reading config file")?;

            let cfg = toml::from_str(&cfg_string)
                .map_err(|e| ParseConfigError::new("config.toml", cfg_string, e))?;

            Ok(cfg)
        }
    }

    pub async fn save(&self) -> Result<()> {
        fs::write(
            &*CFG_FILE_PATH,
            toml::to_string_pretty(&self).map_err(SerializeTomlError::from)?,
        )
        .await
        .into_diagnostic()
        .context("writing config file")?;

        Ok(())
    }

    pub async fn set_default_version(&mut self, default_version: NodeVersion) -> Result<()> {
        self.node.default_version = default_version;
        self.save().await
    }
}
