use std::{ffi::OsString, process::ExitStatus};

use tokio::fs;

use crate::{consts::BIN_DIR, repository::node_path::NodePath};

use self::{mapped_command::MappedCommand, mapped_dir::map_node_bin};
use miette::{IntoDiagnostic, Result};

mod mapped_command;
mod mapped_dir;

/// Responsible for mapping to node executables
/// and managing node versions
pub struct Mapper {
    node_path: NodePath,
}

impl Mapper {
    pub fn new(node_path: NodePath) -> Self {
        Self { node_path }
    }
    /// Executes a mapped command with the given node environment
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn exec(&self, command: String, args: Vec<OsString>) -> Result<ExitStatus> {
        let executable = self.node_path.bin().join(&command);
        let exit_status = MappedCommand::new(command, executable, args).run().await?;
        self.remap_additive().await?;

        Ok(exit_status)
    }

    /// Recreates all environment mappings
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn remap(&self) -> Result<()> {
        fs::remove_dir_all(&*BIN_DIR).await.into_diagnostic()?;
        fs::create_dir_all(&*BIN_DIR).await.into_diagnostic()?;
        self.remap_additive().await?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn remap_additive(&self) -> Result<()> {
        map_node_bin(&self.node_path).await?;

        Ok(())
    }
}
