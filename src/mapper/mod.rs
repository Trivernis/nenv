use std::{env, str::FromStr};

use crate::repository::{NodeVersion, Repository};

use self::error::MapperResult;

pub mod error;
/// Responsible for mapping to node executables
/// and managing node versions
pub struct Mapper {
    repo: Repository,
    active_version: NodeVersion,
}

impl Mapper {
    pub fn new(repository: Repository) -> Self {
        let version =
            Self::get_version().unwrap_or_else(|| repository.config.default_version.to_owned());
        Self {
            repo: repository,
            active_version: version,
        }
    }

    pub fn repository(&self) -> &Repository {
        &self.repo
    }

    /// Sets the given version as the default one
    pub async fn use_version(&mut self, version: &NodeVersion) -> MapperResult<()> {
        self.repo
            .config
            .set_default_version(version.clone())
            .await?;
        self.active_version = version.clone();

        Ok(())
    }

    pub fn active_version(&self) -> &NodeVersion {
        &self.active_version
    }

    fn get_version() -> Option<NodeVersion> {
        env::var("NODE_VERSION")
            .ok()
            .and_then(|v| NodeVersion::from_str(&v).ok())
    }
}
