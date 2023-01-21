use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use tokio::fs::{self, DirEntry};

use crate::{consts::BIN_DIR, repository::node_path::NodePath};

use super::error::MapperResult;

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
        self.write_wrapper_script(&src_path).await
    }

    #[cfg(not(target_os = "windows"))]
    async fn write_wrapper_script(&self, path: &Path) -> MapperResult<()> {
        fs::write(path, format!("#!/bin/sh\nnenv exec {} \"$@\"", self.name)).await?;
        let src_metadata = self.info.metadata().await?;
        fs::set_permissions(&path, src_metadata.permissions()).await?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn write_wrapper_script(&self, path: &Path) -> MapperResult<()> {
        fs::write(
            path.with_extension("bat"),
            format!("nenv exec {} %*", self.name),
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
