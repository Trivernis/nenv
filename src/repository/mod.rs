use core::fmt;
use std::{path::PathBuf, str::FromStr};

use futures::future;
use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::fs;

use crate::{
    config::ConfigAccess,
    consts::{ARCH, BIN_DIR, CACHE_DIR, CFG_DIR, DATA_DIR, NODE_VERSIONS_DIR, OS},
    error::VersionError,
};

use miette::{IntoDiagnostic, Result};

use self::{
    downloader::NodeDownloader,
    node_path::NodePath,
    versions::{SimpleVersion, SimpleVersionInfo, Versions},
};

pub mod downloader;
pub(crate) mod node_path;
pub mod versions;

#[derive(Clone, Debug)]
pub enum NodeVersion {
    Latest,
    LatestLts,
    Lts(String),
    Req(VersionReq),
}

impl FromStr for NodeVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.to_lowercase();

        let version = match &*input {
            "latest" => Self::Latest,
            "lts" => Self::LatestLts,
            _ => {
                let version_string = s.trim().trim_start_matches('v');

                if let Ok(req) = VersionReq::parse(version_string) {
                    Self::Req(req)
                } else {
                    Self::Lts(version_string.to_lowercase())
                }
            }
        };

        Ok(version)
    }
}

impl<'de> Deserialize<'de> for NodeVersion {
    #[inline]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl Serialize for NodeVersion {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl fmt::Display for NodeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeVersion::Latest => String::from("latest"),
            NodeVersion::LatestLts => String::from("lts"),
            NodeVersion::Lts(name) => name.to_owned(),
            NodeVersion::Req(req) => req.to_string(),
        }
        .fmt(f)
    }
}

pub struct Repository {
    versions: Versions,
    downloader: NodeDownloader,
}

impl Repository {
    /// Initializes a new repository with the given confi
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn init(config: ConfigAccess) -> Result<Self> {
        Self::create_folders().await?;
        let downloader = NodeDownloader::new(config.clone());
        let versions = load_versions(&downloader).await?;

        Ok(Self {
            downloader,
            versions,
        })
    }

    #[tracing::instrument(level = "debug")]
    async fn create_folders() -> Result<()> {
        let dirs = vec![
            &*CFG_DIR,
            &*DATA_DIR,
            &*CACHE_DIR,
            &*BIN_DIR,
            &*NODE_VERSIONS_DIR,
        ];
        for result in future::join_all(dirs.into_iter().map(|dir| async move {
            if !dir.exists() {
                fs::create_dir_all(dir).await.into_diagnostic()?;
            }

            Ok(())
        }))
        .await
        {
            #[allow(clippy::question_mark)]
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }

    /// Returns the path for the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_version_path(&self, version: &NodeVersion) -> Result<Option<NodePath>> {
        let info = self.lookup_version(version)?;
        let path = build_version_path(&info.version);

        Ok(if path.exists() {
            Some(NodePath::new(path))
        } else {
            None
        })
    }

    /// Returns a list of installed versions
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn installed_versions(&self) -> Result<Vec<Version>> {
        let mut versions = Vec::new();
        let mut iter = fs::read_dir(&*NODE_VERSIONS_DIR).await.into_diagnostic()?;

        while let Some(entry) = iter.next_entry().await.into_diagnostic()? {
            if let Ok(version) = Version::parse(entry.file_name().to_string_lossy().as_ref()) {
                versions.push(version);
            };
        }

        Ok(versions)
    }

    /// Returns if the given version is installed
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn is_installed(&self, version: &NodeVersion) -> Result<bool> {
        let info = self.lookup_version(version)?;

        Ok(build_version_path(&info.version).exists())
    }

    /// Installs the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn install_version(&self, version: &NodeVersion) -> Result<()> {
        let info = self.lookup_version(version)?;
        self.downloader.download(&info.version).await?;

        Ok(())
    }

    /// Performs a lookup for the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn lookup_version(
        &self,
        version_req: &NodeVersion,
    ) -> Result<&SimpleVersionInfo, VersionError> {
        let version = match version_req {
            NodeVersion::Latest => self.versions.latest(),
            NodeVersion::LatestLts => self.versions.latest_lts(),
            NodeVersion::Lts(lts) => self
                .versions
                .get_lts(lts)
                .ok_or_else(|| VersionError::unknown_version(lts.to_owned()))?,
            NodeVersion::Req(req) => self
                .versions
                .get_fulfilling(req)
                .ok_or_else(|| VersionError::unfulfillable_version(req.to_owned()))?,
        };

        Ok(version)
    }

    /// Returns the reference to all known versions
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn all_versions(&self) -> &Versions {
        &self.versions
    }
}

#[inline]
#[tracing::instrument(level = "debug", skip_all)]
async fn load_versions(downloader: &NodeDownloader) -> Result<Versions> {
    let versions = if let Some(v) = Versions::load().await {
        v
    } else {
        let all_versions = downloader.get_versions().await?;
        let v = Versions::new(all_versions);
        v.save().await?;
        v
    };
    Ok(versions)
}

fn build_version_path(version: &SimpleVersion) -> PathBuf {
    NODE_VERSIONS_DIR
        .join(version.to_string())
        .join(format!("node-v{}-{}-{}", version, OS, ARCH))
}
