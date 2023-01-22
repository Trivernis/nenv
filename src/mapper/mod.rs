use std::{ffi::OsString, process::ExitStatus};

use tokio::fs;

use crate::{
    consts::BIN_DIR,
    error::VersionError,
    repository::{NodeVersion, Repository},
};

use self::{
    mapped_command::MappedCommand,
    mapped_dir::map_node_bin,
    version_detection::{ParallelDetector, VersionDetector},
};
use miette::{IntoDiagnostic, Result};

mod mapped_command;
mod mapped_dir;
mod version_detection;

/// Responsible for mapping to node executables
/// and managing node versions
pub struct Mapper {
    repo: Repository,
    active_version: NodeVersion,
}

impl Mapper {
    pub async fn load(repository: Repository) -> Self {
        let version = Self::get_version()
            .await
            .unwrap_or_else(|| repository.config.node.default_version.to_owned());
        Self {
            repo: repository,
            active_version: version,
        }
    }

    pub fn repository(&self) -> &Repository {
        &self.repo
    }

    /// Sets the given version as the default one
    pub async fn set_default_version(&mut self, version: &NodeVersion) -> Result<()> {
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
    pub async fn exec(&self, command: String, args: Vec<OsString>) -> Result<ExitStatus> {
        let node_path = self
            .repo
            .get_version_path(&self.active_version)?
            .ok_or_else(|| VersionError::not_installed(&self.active_version))?;
        let executable = node_path.bin().join(&command);
        let exit_status = MappedCommand::new(command, executable, args).run().await?;
        self.map_active_version().await?;

        Ok(exit_status)
    }

    /// Recreates all environment mappings
    pub async fn remap(&self) -> Result<()> {
        fs::remove_dir_all(&*BIN_DIR).await.into_diagnostic()?;
        fs::create_dir_all(&*BIN_DIR).await.into_diagnostic()?;
        self.map_active_version().await?;

        Ok(())
    }

    async fn get_version() -> Option<NodeVersion> {
        ParallelDetector::detect_version()
            .await
            .ok()
            .and_then(|v| v)
    }

    /// creates wrapper scripts for the current version
    async fn map_active_version(&self) -> Result<()> {
        let dir = self
            .repo
            .get_version_path(&self.active_version)?
            .ok_or_else(|| VersionError::not_installed(self.active_version.to_string()))?;
        map_node_bin(dir).await?;

        Ok(())
    }
}
