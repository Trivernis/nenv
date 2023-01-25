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
    versioning::{SimpleVersion, VersionMetadata},
};

use miette::{Context, IntoDiagnostic, Result};

use self::{
    downloader::{versions::Versions, NodeDownloader},
    local_versions::InstalledVersions,
    node_path::NodePath,
};

pub mod downloader;
mod local_versions;
pub(crate) mod node_path;

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
    downloader: NodeDownloader,
    installed_versions: InstalledVersions,
}

impl Repository {
    /// Initializes a new repository with the given confi
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn init(config: ConfigAccess) -> Result<Self> {
        Self::create_folders().await?;
        let mut downloader = NodeDownloader::new(config.clone());

        let installed_versions = match InstalledVersions::load() {
            Ok(v) => v,
            Err(_) => {
                let installed: InstalledVersions =
                    load_installed_versions_info(downloader.versions().await?)
                        .await?
                        .into();
                installed.save()?;
                installed
            }
        };

        Ok(Self {
            downloader,
            installed_versions,
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
        future::join_all(dirs.into_iter().map(|dir| async move {
            if !dir.exists() {
                fs::create_dir_all(dir).await?;
            }

            Result::<(), std::io::Error>::Ok(())
        }))
        .await
        .into_iter()
        .fold(Result::Ok(()), |acc, res| acc.and_then(|_| res))
        .into_diagnostic()
        .wrap_err("Failed to create application directory")?;

        Ok(())
    }

    /// Returns the path for the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_version_path(&mut self, version: &NodeVersion) -> Result<Option<NodePath>> {
        let info = if let Ok(i) = self.lookup_local_version(version) {
            i
        } else {
            self.lookup_remote_version(version).await?
        };
        let path = build_version_path(&info.version);

        Ok(if path.exists() {
            Some(NodePath::new(path))
        } else {
            None
        })
    }

    /// Returns a list of installed versions
    pub fn installed_versions(&self) -> Vec<Version> {
        self.installed_versions
            .all()
            .into_iter()
            .map(|v| v.clone().into())
            .collect()
    }

    /// Returns if the given version is installed
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn is_installed(&mut self, version: &NodeVersion) -> Result<bool> {
        let info = if let Ok(v) = self.lookup_local_version(version) {
            v
        } else {
            self.lookup_remote_version(version).await?
        };

        Ok(build_version_path(&info.version).exists())
    }

    /// Installs the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn install_version(&mut self, version: &NodeVersion) -> Result<()> {
        let info = self.lookup_remote_version(version).await?.to_owned();
        self.downloader.download(&info.version).await?;
        self.installed_versions.insert((info.version, info));
        self.installed_versions.save()?;

        Ok(())
    }

    /// Uninstalls the given node version by deleting the versions directory
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn uninstall(&mut self, version: &NodeVersion) -> Result<()> {
        let info = self.lookup_local_version(version)?.clone();
        let version_dir = NODE_VERSIONS_DIR.join(info.version.to_string());

        if !version_dir.exists() {
            return Err(VersionError::not_installed(version).into());
        }

        fs::remove_dir_all(version_dir)
            .await
            .into_diagnostic()
            .context("Deleting node version")?;
        self.installed_versions.remove(&info.version);
        self.installed_versions.save()?;

        Ok(())
    }

    /// Performs a lookup for the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn lookup_remote_version(
        &mut self,
        version_req: &NodeVersion,
    ) -> Result<&VersionMetadata> {
        let versions = self.downloader.versions().await?;

        let version = match version_req {
            NodeVersion::Latest => versions.latest(),
            NodeVersion::LatestLts => versions.latest_lts(),
            NodeVersion::Lts(lts) => versions
                .get_lts(lts)
                .ok_or_else(|| VersionError::unknown_version(lts.to_owned()))?,
            NodeVersion::Req(req) => versions
                .get_fulfilling(req)
                .ok_or_else(|| VersionError::unfulfillable_version(req.to_owned()))?,
        };

        Ok(version)
    }

    /// Performs a lookup for the given node version
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn lookup_local_version(&self, version_req: &NodeVersion) -> Result<&VersionMetadata> {
        let versions = &self.installed_versions;
        let version = match version_req {
            NodeVersion::Lts(lts) => versions
                .lts(lts)
                .ok_or_else(|| VersionError::unknown_version(lts.to_owned()))?,
            NodeVersion::Req(req) => versions
                .fulfilling(req)
                .ok_or_else(|| VersionError::unfulfillable_version(req.to_owned()))?,
            _ => return Err(VersionError::unsupported(version_req.to_owned()).into()),
        };

        Ok(version)
    }

    /// Returns the reference to all known versions
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn all_versions(&mut self) -> Result<&Versions> {
        self.downloader.versions().await
    }
}

fn build_version_path(version: &SimpleVersion) -> PathBuf {
    NODE_VERSIONS_DIR
        .join(version.to_string())
        .join(format!("node-v{}-{}-{}", version, OS, ARCH))
}

async fn load_installed_versions_info(versions: &Versions) -> Result<Vec<VersionMetadata>> {
    let mut installed_versions = Vec::new();
    let mut iter = fs::read_dir(&*NODE_VERSIONS_DIR).await.into_diagnostic()?;

    while let Some(entry) = iter.next_entry().await.into_diagnostic()? {
        if let Ok(version) = Version::parse(entry.file_name().to_string_lossy().as_ref()) {
            installed_versions.push(version);
        };
    }
    let versions = installed_versions
        .into_iter()
        .filter_map(|v| versions.get(&v))
        .cloned()
        .collect();

    Ok(versions)
}
