use core::fmt;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::{
    fs::{self, File},
    io::BufWriter,
};

use crate::{
    consts::{
        ARCH, BIN_DIR, CACHE_DIR, CFG_DIR, DATA_DIR, NODE_ARCHIVE_SUFFIX, NODE_VERSIONS_DIR, OS,
    },
    error::LibResult,
    web_api::{VersionInfo, WebApi},
};

use self::{config::Config, node_paths::NodePath, versions::Versions};

pub mod config;
pub(crate) mod extract;
mod node_paths;
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
                if let Ok(req) = VersionReq::parse(s) {
                    Self::Req(req)
                } else {
                    Self::Lts(s.to_lowercase())
                }
            }
        };

        Ok(version)
    }
}

impl NodeVersion {
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }

    pub fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
    web_api: WebApi,
    pub config: Config,
}

impl Repository {
    /// Initializes a new repository with the given confi
    pub async fn init(config: Config) -> LibResult<Self> {
        Self::create_folders().await?;
        let web_api = WebApi::new(&config.dist_base_url);
        let all_versions = web_api.get_versions().await?;

        Ok(Self {
            config,
            web_api,
            versions: Versions::new(all_versions),
        })
    }

    async fn create_folders() -> LibResult<()> {
        let dirs = vec![
            &*CFG_DIR,
            &*DATA_DIR,
            &*CACHE_DIR,
            &*BIN_DIR,
            &*NODE_VERSIONS_DIR,
        ];
        for dir in dirs {
            if !dir.exists() {
                fs::create_dir_all(dir).await?;
            }
        }

        Ok(())
    }

    /// Returns the path for the given node version
    pub async fn get_version_path(&self, version: &NodeVersion) -> LibResult<Option<NodePath>> {
        let info = self.parse_req(&version);
        let mut iter = fs::read_dir(&*NODE_VERSIONS_DIR).await?;
        let mut path = None;

        while let Some(entry) = iter.next_entry().await? {
            if let Ok(version) = Version::parse(entry.file_name().to_string_lossy().as_ref()) {
                if version == info.version {
                    path = Some(entry.path());
                }
            };
        }
        let Some(path) = path else {
            return Ok(None);
        };
        let path = path.join(format!("node-v{}-{}-{}", info.version, OS, ARCH));

        Ok(Some(NodePath::new(path)))
    }

    /// Returns a list of installed versions
    pub async fn installed_versions(&self) -> LibResult<Vec<Version>> {
        let mut versions = Vec::new();
        let mut iter = fs::read_dir(&*NODE_VERSIONS_DIR).await?;

        while let Some(entry) = iter.next_entry().await? {
            if let Ok(version) = Version::parse(entry.file_name().to_string_lossy().as_ref()) {
                versions.push(version);
            };
        }

        Ok(versions)
    }

    /// Returns if the given version is installed
    pub async fn is_installed(&self, version: &NodeVersion) -> LibResult<bool> {
        let info = self.parse_req(version);

        Ok(self.installed_versions().await?.contains(&info.version))
    }

    /// Installs a specified node version
    pub async fn install_version(&self, version_req: &NodeVersion) -> LibResult<()> {
        let info = self.parse_req(&version_req);
        let archive_path = self.download_version(&info.version).await?;
        self.extract_archive(info, &archive_path)?;

        Ok(())
    }

    async fn download_version(&self, version: &Version) -> LibResult<PathBuf> {
        let download_path = CACHE_DIR.join(format!("node-v{}{}", version, *NODE_ARCHIVE_SUFFIX));

        if download_path.exists() {
            return Ok(download_path);
        }
        let mut download_writer = BufWriter::new(File::create(&download_path).await?);
        self.web_api
            .download_version(version.to_string(), &mut download_writer)
            .await?;

        Ok(download_path)
    }

    fn extract_archive(&self, info: &VersionInfo, archive_path: &Path) -> LibResult<()> {
        let dst_path = NODE_VERSIONS_DIR.join(info.version.to_string());
        extract::extract_file(archive_path, &dst_path)?;

        Ok(())
    }

    fn parse_req(&self, version_req: &NodeVersion) -> &VersionInfo {
        match version_req {
            NodeVersion::Latest => self.versions.latest(),
            NodeVersion::LatestLts => self.versions.latest_lts(),
            NodeVersion::Lts(lts) => self.versions.get_lts(&lts).expect("Version not found"),
            NodeVersion::Req(req) => self
                .versions
                .get_fulfilling(&req)
                .expect("Version not found"),
        }
    }
}
