use std::{env, ffi::OsString, process::ExitStatus, str::FromStr};

use tokio::fs;

use crate::{
    consts::BIN_DIR,
    error::VersionError,
    repository::{NodeVersion, Repository},
};

use self::{
    error::MapperError, mapped_command::MappedCommand, mapped_dir::map_node_bin,
    package_info::PackageInfo,
};
use miette::{IntoDiagnostic, Result};

pub mod error;
mod mapped_command;
mod mapped_dir;
mod package_info;

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
            .unwrap_or_else(|| repository.config.default_version.to_owned());
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
        let executable = node_path.bin().join(command);
        let exit_status = MappedCommand::new(executable, args)
            .run()
            .await
            .map_err(MapperError::from)?;
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
        if let Some(version) = PackageInfo::find()
            .await
            .ok()
            .and_then(|i| i)
            .and_then(|i| i.engines)
            .and_then(|e| e.node)
        {
            Some(NodeVersion::Req(version))
        } else {
            env::var("NODE_VERSION")
                .ok()
                .and_then(|v| NodeVersion::from_str(&v).ok())
        }
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
