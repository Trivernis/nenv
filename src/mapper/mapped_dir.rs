use std::{collections::HashSet, io, path::Path};

use tokio::fs::{self, DirEntry};

use crate::{consts::BIN_DIR, repository::node_path::NodePath};

use super::error::{MapperError, MapperResult};

struct NodeApp {
    info: DirEntry,
    name: String,
}

impl NodeApp {
    pub fn new(info: DirEntry) -> Self {
        let path = info.path();
        let name = path.file_stem().unwrap();
        let name = name.to_string_lossy().into_owned();

        Self { info, name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    /// creates wrappers to map this application
    pub async fn map_executable(&self) -> MapperResult<()> {
        let src_path = BIN_DIR.join(self.info.file_name());
        self.write_wrapper_script(&src_path)
            .await
            .map_err(|err| MapperError::DirMapping { src: src_path, err })
    }

    #[cfg(not(target_os = "windows"))]
    async fn write_wrapper_script(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(path, format!("#!/bin/sh\nnenv exec {} \"$@\"", self.name)).await?;
        let src_metadata = self.info.metadata().await?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn write_wrapper_script(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(
            path.with_extension("bat"),
            format!("@echo off\nnenv exec {} %*", self.name),
        )
        .await?;
        let src_metadata = self.info.metadata().await?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }
}

pub async fn map_node_bin(node_path: NodePath) -> MapperResult<()> {
    let mapped_app_names = get_applications(&*BIN_DIR)
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

async fn get_applications(path: &Path) -> MapperResult<Vec<NodeApp>> {
    let mut files = Vec::new();
    let mut iter = fs::read_dir(path)
        .await
        .map_err(|err| MapperError::DirMapping {
            src: path.to_owned(),
            err,
        })?;

    while let Some(entry) = iter.next_entry().await? {
        let entry_path = entry.path();

        if entry_path.is_file() || !exclude_path(&entry_path) {
            files.push(NodeApp::new(entry));
        }
    }

    Ok(files)
}

#[cfg(not(target_os = "windows"))]
fn exclude_path(_path: &Path) -> bool {
    false
}

#[cfg(target_os = "windows")]
fn exclude_path(path: &Path) -> bool {
    let Some(extension) = path.extension() else {
        return true;
    };
    let extension = extension.to_string_lossy();
    let allowed_extensions = ["exe", "bat", "cmd", "ps1"];

    allowed_extensions.contains(&extension.as_ref())
}
