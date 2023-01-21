use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use tokio::fs::{self, DirEntry};

use crate::{consts::BIN_DIR, repository::node_path::NodePath};

use super::error::MapperResult;

struct NodeApp {
    info: DirEntry,
}

impl NodeApp {
    pub fn new(info: DirEntry) -> Self {
        Self { info }
    }

    /// returns the name of the application
    pub fn name(&self) -> String {
        let name = self.info.file_name();
        name.to_string_lossy().into_owned()
    }

    pub async fn unmap(&self) -> MapperResult<()> {
        fs::remove_file(self.info.path()).await?;

        Ok(())
    }

    /// creates wrappers to map this application
    pub async fn map_executable(&self) -> MapperResult<()> {
        let src_path = BIN_DIR.join(self.info.file_name());
        let name = self.info.file_name();
        let name = name.to_string_lossy();
        self.write_wrapper_script(&name, &src_path).await
    }

    #[cfg(not(target_os = "windows"))]
    async fn write_wrapper_script(&self, name: &str, path: &Path) -> MapperResult<()> {
        fs::write(
            path,
            format!(
                r#"#!/bin/sh
                nenv exec {name} "$@""#
            ),
        )
        .await?;
        let src_metadata = self.info.metadata().await?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn write_wrapper_script(&self, name: &str, path: &Path) -> MapperResult<()> {
        fs::write(path, format!("nenv exec {name} %*")).await?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }
}

pub async fn map_node_bin(node_path: NodePath) -> MapperResult<()> {
    let applications = get_applications(&node_path.bin()).await?;
    let mapped_applications = get_applications(&*BIN_DIR).await?;
    let mut new_mapped = HashSet::new();

    for application in applications {
        application.map_executable().await?;
        new_mapped.insert(application.name());
    }
    for app in mapped_applications {
        if !new_mapped.contains(&app.name()) {
            app.unmap().await?;
        }
    }

    Ok(())
}

async fn get_applications(path: &PathBuf) -> MapperResult<Vec<NodeApp>> {
    let mut files = Vec::new();
    let mut iter = fs::read_dir(path).await?;

    while let Some(entry) = iter.next_entry().await? {
        if entry.path().is_file() {
            files.push(NodeApp::new(entry));
        }
    }

    Ok(files)
}
