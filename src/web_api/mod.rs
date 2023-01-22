use std::{
    cmp::min,
    fmt::{Debug, Display},
};

use crate::{
    consts::{NODE_ARCHIVE_SUFFIX, NODE_DIST_URL},
    error::ReqwestError,
    utils::progress_bar,
};

use reqwest::Client;

mod model;
use futures_util::StreamExt;
use miette::{miette, Context, IntoDiagnostic, Result};
pub use model::*;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
pub struct WebApi {
    base_url: String,
    client: Client,
}

impl Default for WebApi {
    fn default() -> Self {
        Self::new(NODE_DIST_URL)
    }
}

impl WebApi {
    /// Creates a new instance to access the nodejs website
    pub fn new<S: ToString>(base_url: S) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    /// Returns the list of available node versions
    #[tracing::instrument(level = "trace")]
    pub async fn get_versions(&self) -> Result<Vec<VersionInfo>> {
        let versions = self
            .client
            .get(format!("{}/index.json", self.base_url))
            .send()
            .await
            .map_err(ReqwestError::from)
            .context("Fetching versions")?
            .json()
            .await
            .map_err(ReqwestError::from)
            .context("Parsing versions response")?;

        Ok(versions)
    }

    /// Downloads a specific node version
    /// and writes it to the given writer
    #[tracing::instrument(level = "trace", skip(writer))]
    pub async fn download_version<W: AsyncWrite + Unpin, S: Display + Debug>(
        &self,
        version: S,
        writer: &mut W,
    ) -> Result<u64> {
        let res = self
            .client
            .get(format!(
                "{}/v{version}/node-v{version}{}",
                self.base_url, *NODE_ARCHIVE_SUFFIX
            ))
            .send()
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
}
