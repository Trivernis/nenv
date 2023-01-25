use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
};

use miette::miette;
use tokio::fs;

use crate::{consts::BIN_DIR, error::MapDirError, repository::node_path::NodePath};

use miette::{Context, IntoDiagnostic, Result};
pub struct NodeApp {
    path: PathBuf,
    name: String,
}

impl NodeApp {
    pub fn new(path: PathBuf) -> Self {
        let name = path.file_stem().unwrap();
        let name = name.to_string_lossy().into_owned();

        Self { path, name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    /// creates wrappers to map this application
    pub async fn map_executable(&self) -> Result<()> {
        let src_path = BIN_DIR.join(
            self.path
                .file_name()
                .ok_or_else(|| miette!("The given path is not a file."))?,
        );
        self.write_wrapper_script(&src_path)
            .await
            .into_diagnostic()
            .context("Creating executable wrapper script")
    }

    #[cfg(not(windows))]
    async fn write_wrapper_script(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(path, format!("#!/bin/sh\nnenv exec {} \"$@\"", self.name)).await?;
        let src_metadata = self.path.metadata()?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }

    #[cfg(windows)]
    async fn write_wrapper_script(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(
            path.with_extension("bat"),
            format!("@echo off\nnenv exec {} %*", self.name),
        )
        .await?;
        let src_metadata = self.path.metadata()?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }
}

pub async fn map_direct(paths: Vec<PathBuf>) -> Result<()> {
    let results = futures::future::join_all(
        paths
            .into_iter()
            .map(NodeApp::new)
            .map(|n| async move { n.map_executable().await }),
    )
    .await;
    results
        .into_iter()
        .fold(Result::Ok(()), |acc, res| acc.and(res))?;

    Ok(())
}

pub async fn map_node_bin(node_path: &NodePath) -> Result<()> {
    let mapped_app_names = get_applications(&BIN_DIR)
        .await?
        .iter()
        .map(NodeApp::name)
        .cloned()
        .collect::<HashSet<_>>();

    let mut applications = get_applications(&node_path.bin()).await?;
    applications.retain(|app| !mapped_app_names.contains(app.name()));

    futures::future::join_all(applications.iter().map(NodeApp::map_executable)).await;

    Ok(())
}

async fn get_applications(path: &Path) -> Result<Vec<NodeApp>> {
    let mut files = Vec::new();
    let mut iter = fs::read_dir(path).await.map_err(|err| MapDirError {
        dir: path.to_owned(),
        caused_by: err,
    })?;

    while let Some(entry) = iter
        .next_entry()
        .await
        .into_diagnostic()
        .context("Reading directory entries")?
    {
        let entry_path = entry.path();

        if entry_path.is_file() && !exclude_path(&entry_path) {
            files.push(NodeApp::new(entry.path()));
        }
    }

    Ok(files)
}

#[cfg(not(windows))]
fn exclude_path(_path: &Path) -> bool {
    false
}

#[cfg(windows)]
fn exclude_path(path: &Path) -> bool {
    let Some(extension) = path.extension() else {
        return true;
    };
    let extension = extension.to_string_lossy();
    let allowed_extensions = ["exe", "bat", "cmd", "ps1"];

    !allowed_extensions.contains(&extension.as_ref())
}
