use miette::Context;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::SerializeTomlError;
use crate::{
    consts::{CFG_DIR, CFG_FILE_PATH, NODE_DIST_URL},
    error::ParseTomlError,
};

use super::NodeVersion;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub dist_base_url: String,
    #[serde(with = "NodeVersion")]
    pub default_version: NodeVersion,
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
                .map_err(|e| ParseTomlError::new("config.toml", cfg_string, e))?;

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
        self.default_version = default_version;
        self.save().await
    }
}
