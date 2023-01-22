use std::str::FromStr;

use miette::{Context, IntoDiagnostic};

use crate::repository::NodeVersion;

use super::VersionDetector;

pub struct EnvDetector;

#[async_trait::async_trait]
impl VersionDetector for EnvDetector {
    async fn detect_version() -> miette::Result<Option<crate::repository::NodeVersion>> {
        std::env::var("NODE_VERSION")
            .into_diagnostic()
            .context("Reading version from environment")
            .map(|v| NodeVersion::from_str(&v).ok())
    }
}
