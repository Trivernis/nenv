use std::{collections::HashMap, path::Path};

use miette::{IntoDiagnostic, NamedSource, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;

use crate::{error::ParseJsonError, repository::NodeVersion, utils::find_in_parents};

use super::VersionDetector;

pub struct PackageJsonDetector;

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

#[async_trait::async_trait]
impl VersionDetector for PackageJsonDetector {
    async fn detect_version() -> Result<Option<crate::repository::NodeVersion>> {
        Ok(PackageInfo::find()
            .await?
            .and_then(|p| p.engines)
            .and_then(|e| e.node)
            .map(NodeVersion::Req))
    }
}

impl PackageInfo {
    pub async fn find() -> Result<Option<Self>> {
        let dir = std::env::current_dir().into_diagnostic()?;

        if let Some(path) = find_in_parents(dir, "package.json") {
            let info = Self::load(&path).await?;
            Ok(Some(info))
        } else {
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
