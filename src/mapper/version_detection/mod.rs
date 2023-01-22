use async_trait::async_trait;

use futures::future;
use miette::Result;
mod env_detector;
mod package_json_detector;
mod version_file_detector;

use crate::repository::NodeVersion;

use self::{
    env_detector::EnvDetector, package_json_detector::PackageJsonDetector,
    version_file_detector::VersionFileDetector,
};

#[async_trait]
pub trait VersionDetector {
    async fn detect_version() -> Result<Option<NodeVersion>>;
}

pub struct ParallelDetector;

#[async_trait]
impl VersionDetector for ParallelDetector {
    async fn detect_version() -> Result<Option<NodeVersion>> {
        let version = future::join_all(vec![
            VersionFileDetector::detect_version(),
            PackageJsonDetector::detect_version(),
            EnvDetector::detect_version(),
        ])
        .await
        .into_iter()
        .filter_map(Result::ok)
        .find_map(|v| v);

        Ok(version)
    }
}
