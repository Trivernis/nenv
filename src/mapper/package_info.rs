use std::{collections::HashMap, path::Path};

use miette::{IntoDiagnostic, NamedSource, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;

use crate::error::ParseJsonError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageInfo {
    pub engines: Option<EngineInfo>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EngineInfo {
    pub node: Option<VersionReq>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl PackageInfo {
    pub async fn find() -> Result<Option<Self>> {
        let mut dir = std::env::current_dir().into_diagnostic()?;
        let file_path = dir.join("package.json");

        if file_path.exists() {
            let info = Self::load(&file_path).await?;

            Ok(Some(info))
        } else {
            while let Some(parent) = dir.parent() {
                dir = parent.to_owned();
                let file_path = dir.join("package.json");

                if file_path.exists() {
                    let info = Self::load(&file_path).await?;
                    return Ok(Some(info));
                }
            }
            Ok(None)
        }
    }

    /// Loads the package.json config file
    pub async fn load(path: &Path) -> Result<Self> {
        let file_content = fs::read_to_string(&path).await.into_diagnostic()?;

        let cfg = serde_json::from_str(&file_content).map_err(|e| ParseJsonError {
            src: NamedSource::new(path.file_name().unwrap().to_string_lossy(), file_content),
            pos: (e.column(), e.column()).into(),
            caused_by: e,
        })?;

        Ok(cfg)
    }
}
