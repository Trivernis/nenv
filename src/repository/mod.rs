use std::path::PathBuf;

use semver::{Version, VersionReq};
use tokio::{
    fs::{self, File},
    io::BufWriter,
};

use crate::{
    consts::{BIN_DIR, CACHE_DIR, CFG_DIR, DATA_DIR, NODE_ARCHIVE_SUFFIX, NODE_VERSIONS_DIR},
    error::LibResult,
    web_api::{VersionInfo, WebApi},
};

use self::{config::Config, versions::Versions};

pub mod config;
pub mod versions;

pub enum NodeVersion {
    Latest,
    LatestLts,
    Lts(String),
    Req(VersionReq),
}

pub struct Repository {
    versions: Versions,
    web_api: WebApi,
    config: Config,
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

    pub async fn install_version(&self, version_req: NodeVersion) -> LibResult<()> {
        let info = self.parse_req(version_req);
        let archive_path = self.download_version(&info.version).await?;

        todo!()
    }

    async fn download_version(&self, version: &Version) -> LibResult<PathBuf> {
        let download_path = CACHE_DIR.join(format!("node-v{}{}", version, *NODE_ARCHIVE_SUFFIX));
        let mut download_writer = BufWriter::new(File::create(&download_path).await?);
        self.web_api
            .download_version(version.to_string(), &mut download_writer)
            .await?;

        Ok(download_path)
    }

    fn parse_req(&self, version_req: NodeVersion) -> &VersionInfo {
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
