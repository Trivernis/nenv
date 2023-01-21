use std::collections::HashMap;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{consts::VERSION_FILE_PATH, error::LibResult, web_api::VersionInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Versions {
    lts_versions: HashMap<String, VersionReq>,
    versions: HashMap<Version, VersionInfo>,
    sorted_versions: Vec<Version>,
}

impl Versions {
    /// Loads the versions from the cached versions.json file
    pub(crate) async fn load() -> Option<Self> {
        if !VERSION_FILE_PATH.exists() {
            return None;
        }
        let versions_string = fs::read_to_string(&*VERSION_FILE_PATH).await.ok()?;
        let versions = serde_json::from_str(&versions_string).ok()?;

        Some(versions)
    }

    /// creates a new instance to access version information
    pub fn new(all_versions: Vec<VersionInfo>) -> Self {
        let lts_versions = all_versions
            .iter()
            .filter_map(|v| {
                Some((
                    v.lts.as_ref()?.to_lowercase(),
                    VersionReq::parse(&format!("{}", v.version.major)).ok()?,
                ))
            })
            .collect::<HashMap<_, _>>();
        let mut sorted_versions = all_versions
            .iter()
            .map(|v| v.version.to_owned())
            .collect::<Vec<_>>();
        sorted_versions.sort();

        let versions = all_versions
            .into_iter()
            .map(|v| (v.version.to_owned(), v))
            .collect::<HashMap<_, _>>();

        Self {
            lts_versions,
            versions,
            sorted_versions,
        }
    }

    pub(crate) async fn save(&self) -> LibResult<()> {
        let json_string = serde_json::to_string(&self)?;
        fs::write(&*VERSION_FILE_PATH, json_string).await?;

        Ok(())
    }

    /// Returns the latest known node version
    pub fn latest(&self) -> &VersionInfo {
        self.versions
            .get(self.sorted_versions.last().expect("No known node versions"))
            .unwrap()
    }

    /// Returns the latest node lts version
    pub fn latest_lts(&self) -> &VersionInfo {
        let mut versions = self
            .lts_versions
            .values()
            .filter_map(|req| self.get_fulfilling(req))
            .collect::<Vec<_>>();
        versions.sort_by_key(|v| &v.version);
        versions.last().expect("No known lts node versions")
    }

    /// Returns a lts version by name
    pub fn get_lts<S: AsRef<str>>(&self, lts_name: S) -> Option<&VersionInfo> {
        let lts_version = self.lts_versions.get(lts_name.as_ref())?;
        self.get_fulfilling(lts_version)
    }

    /// Returns any version that fulfills the given requirement
    pub fn get_fulfilling(&self, req: &VersionReq) -> Option<&VersionInfo> {
        let fulfilling_versions = self
            .sorted_versions
            .iter()
            .filter(|v| req.matches(v))
            .collect::<Vec<_>>();

        self.versions.get(fulfilling_versions.last()?)
    }

    /// Returns the info for the given version
    pub fn get(&self, version: &Version) -> Option<&VersionInfo> {
        self.versions.get(version)
    }
}
