use std::{env, ffi::OsString, process::ExitStatus};

use envmnt::ListOptions;
use tokio::fs;

use crate::{
    consts::{BIN_DIR, SEARCH_PATH_SEPARATOR},
    repository::node_path::NodePath,
};

use self::{
    mapped_command::MappedCommand,
    mapped_dir::{map_direct, map_node_bin},
};
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
        self.set_env();
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

    /// Maps all binaries
    pub async fn map_bins(&self, binaries: Vec<(String, NodePath)>) -> Result<()> {
        map_direct(
            binaries
                .into_iter()
                .map(|(cmd, path)| path.bin().join(cmd))
                .collect(),
        )
        .await
    }

    fn set_env(&self) {
        env::set_var(
            "NODE_PATH",
            self.node_path.node_modules().to_string_lossy().to_string(),
        );
        let list_options = ListOptions {
            separator: Some(SEARCH_PATH_SEPARATOR.to_string()),
            ignore_empty: true,
        };
        let mut path_env = envmnt::get_list_with_options("PATH", &list_options).unwrap_or_default();
        path_env.insert(0, self.node_path.bin().to_string_lossy().to_string());
        envmnt::set_list_with_options("PATH", &path_env, &list_options);
    }
}
