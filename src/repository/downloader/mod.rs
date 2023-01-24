use std::{
    cmp::min,
    fmt::Debug,
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{
    config::ConfigAccess,
    consts::{CACHE_DIR, NODE_ARCHIVE_SUFFIX, NODE_VERSIONS_DIR},
    error::ReqwestError,
    utils::progress_bar,
    versioning::SimpleVersion,
};

use self::versions::Versions;

use futures::StreamExt;
use miette::{miette, Context, IntoDiagnostic, Result};
use tokio::{
    fs::File,
    io::{AsyncWrite, AsyncWriteExt, BufWriter},
};
mod extract;
mod version_info;
pub mod versions;
pub use version_info::VersionInfo;

#[derive(Clone)]
pub struct NodeDownloader {
    config: ConfigAccess,
    versions: Option<Versions>,
}

impl NodeDownloader {
    pub fn new(config: ConfigAccess) -> Self {
        Self {
            config,
            versions: None,
        }
    }

    /// Returns the list of available node versions
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn versions(&mut self) -> Result<&Versions> {
        if self.versions.is_none() {
            self.versions = Some(self.load_versions().await?);
        }

        Ok(self.versions.as_ref().unwrap())
    }

    async fn load_versions(&self) -> Result<Versions> {
        let versions = if let Some(v) = Versions::load().await {
            v
        } else {
            let versions = reqwest::get(format!("{}/index.json", self.base_url().await))
                .await
                .map_err(ReqwestError::from)
                .context("Fetching versions")?
                .json()
                .await
                .map_err(ReqwestError::from)
                .context("Parsing versions response")?;
            let v = Versions::new(versions);
            v.save().await?;
            v
        };

        Ok(versions)
    }

    /// Downloads a specified node version to the repository
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn download(&self, version: &SimpleVersion) -> Result<()> {
        let archive_path = self.download_archive_to_cache(version).await?;
        self.extract_archive(version, &archive_path)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn extract_archive(&self, version: &SimpleVersion, archive_path: &Path) -> Result<()> {
        let dst_path = NODE_VERSIONS_DIR.join(version.to_string());
        extract::extract_file(archive_path, &dst_path)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn download_archive_to_cache(&self, version: &SimpleVersion) -> Result<PathBuf> {
        let download_path = CACHE_DIR.join(format!("node-v{}{}", version, *NODE_ARCHIVE_SUFFIX));

        if download_path.exists() {
            return Ok(download_path);
        }
        let mut download_writer =
            BufWriter::new(File::create(&download_path).await.into_diagnostic()?);
        self.download_archive(version.to_string(), &mut download_writer)
            .await?;

        Ok(download_path)
    }
    /// Downloads a specific node version
    /// and writes it to the given writer
    #[tracing::instrument(level = "debug", skip(self, writer))]
    pub async fn download_archive<W: AsyncWrite + Unpin, S: Display + Debug>(
        &self,
        version: S,
        writer: &mut W,
    ) -> Result<u64> {
        let res = reqwest::get(format!(
            "{}/v{version}/node-v{version}{}",
            self.base_url().await,
            *NODE_ARCHIVE_SUFFIX
        ))
        .await
        .map_err(ReqwestError::from)
        .context("Downloading nodejs")?;

        let total_size = res
            .content_length()
            .ok_or_else(|| miette!("Missing content_length header"))?;

        let pb = progress_bar(total_size);
        pb.set_message(format!("Downloading node v{version}"));
        let mut stream = res.bytes_stream();
        let mut total_downloaded = 0;

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(ReqwestError::from)?;
            writer
                .write_all(&chunk)
                .await
                .into_diagnostic()
                .context("Writing download chunk to file")?;
            total_downloaded = min(chunk.len() as u64 + total_downloaded, total_size);
            pb.set_position(total_downloaded);
        }

        writer.flush().await.into_diagnostic()?;
        pb.finish_with_message(format!("Downloaded node v{version}."));

        Ok(total_downloaded)
    }

    async fn base_url(&self) -> String {
        self.config.get().await.download.dist_base_url.to_owned()
    }
}
