use std::{env, ffi::OsString, process::ExitStatus, str::FromStr};

use crate::{
    error::LibResult,
    repository::{NodeVersion, Repository},
};

use self::{error::MapperError, mapped_command::MappedCommand, mapped_dir::map_node_bin};

pub mod error;
mod mapped_command;
mod mapped_dir;
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
    pub async fn use_version(&mut self, version: &NodeVersion) -> LibResult<()> {
        self.repo
            .config
            .set_default_version(version.clone())
            .await?;
        self.active_version = version.clone();
        self.map_active_version().await?;

        Ok(())
    }

    pub fn active_version(&self) -> &NodeVersion {
        &self.active_version
    }

    /// Executes a mapped command with the given node environment
    pub async fn exec(&self, command: String, args: Vec<OsString>) -> LibResult<ExitStatus> {
        self.map_active_version().await?;
        let node_path = self
            .repo
            .get_version_path(&self.active_version)
            .await?
            .expect("version not installed");
        let executable = node_path.bin().join(command);
        let exit_status = MappedCommand::new(executable, args)
            .run()
            .await
            .map_err(MapperError::from)?;

        Ok(exit_status)
    }

    fn get_version() -> Option<NodeVersion> {
        env::var("NODE_VERSION")
            .ok()
            .and_then(|v| NodeVersion::from_str(&v).ok())
    }

    /// creates wrapper scripts for the current version
    async fn map_active_version(&self) -> LibResult<()> {
        let dir = self
            .repo
            .get_version_path(&self.active_version)
            .await?
            .expect("missing version");
        map_node_bin(dir).await?;

        Ok(())
    }
}
