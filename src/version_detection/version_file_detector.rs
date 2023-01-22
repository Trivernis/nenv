use std::str::FromStr;

use miette::{Context, IntoDiagnostic};
use tokio::fs;

use crate::{repository::NodeVersion, utils::find_in_parents};

use super::VersionDetector;

pub struct VersionFileDetector;

#[async_trait::async_trait]
impl VersionDetector for VersionFileDetector {
    async fn detect_version() -> miette::Result<Option<crate::repository::NodeVersion>> {
        let dir = std::env::current_dir().into_diagnostic()?;

        if let Some(path) = find_in_parents(dir, ".node-version") {
            let version_string = fs::read_to_string(path)
                .await
                .into_diagnostic()
                .context("Reading version file.")?;
            let version = version_string
                .lines()
                .into_iter()
                .find_map(|l| NodeVersion::from_str(l).ok());

            Ok(version)
        } else {
            Ok(None)
        }
    }
}
