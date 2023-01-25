use std::collections::HashMap;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio::fs;

use crate::{
    consts::VERSION_FILE_PATH,
    error::SerializeBincodeError,
    versioning::{SimpleVersion, VersionMetadata},
};
use miette::{Context, IntoDiagnostic, Result};

use super::VersionInfo;

#[derive(Clone, Serialize, Deserialize)]
pub struct Versions {
    lts_versions: HashMap<String, u8>,
    versions: HashMap<SimpleVersion, VersionMetadata>,
    // as this field is not serialized
    // it needs to be calculated after serialization
    #[serde(skip)]
    sorted_versions: Vec<SimpleVersion>,
}

impl Versions {
    /// Loads the versions from the cached versions.json file
    pub(crate) async fn load() -> Option<Self> {
        if !VERSION_FILE_PATH.exists() {
            return None;
        }
        let byte_contents = fs::read(&*VERSION_FILE_PATH).await.ok()?;

        match bincode::deserialize::<Versions>(&byte_contents) {
            Ok(mut versions) => {
                // create the list of sorted versions
                // this is faster when done directly rather than
                // storing it
                versions.create_sorted_versions();
                Some(versions)
            }
            Err(e) => {
                tracing::error!("Failed to deserialize cache {e}");
                fs::remove_file(&*VERSION_FILE_PATH).await.ok()?;
                None
            }
        }
    }

    /// creates a new instance to access version information
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(all_versions: Vec<VersionInfo>) -> Self {
        let lts_versions = all_versions
            .iter()
            .filter_map(|v| Some((v.lts.lts_ref()?.to_lowercase(), v.version.major as u8)))
            .collect::<HashMap<_, _>>();
        let mut sorted_versions = all_versions
            .iter()
            .map(|v| v.version.to_owned().into())
            .collect::<Vec<_>>();
        sorted_versions.sort();

        let versions = all_versions
            .into_iter()
            .map(|v| (v.version.to_owned().into(), v.into()))
            .collect::<HashMap<_, _>>();

        Self {
            lts_versions,
            versions,
            sorted_versions,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn save(&self) -> Result<()> {
        let byte_content = bincode::serialize(self).map_err(SerializeBincodeError::from)?;
        fs::write(&*VERSION_FILE_PATH, byte_content)
            .await
            .into_diagnostic()
            .context("Caching available node version.")?;

        Ok(())
    }

    /// Returns the latest known node version
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn latest(&self) -> &VersionMetadata {
        self.versions
            .get(self.sorted_versions.last().expect("No known node versions"))
            .unwrap()
    }

    /// Returns the latest node lts version
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn latest_lts(&self) -> &VersionMetadata {
        let mut versions = self
            .lts_versions
            .values()
            .filter_map(|req| self.get_latest_for_major(*req))
            .collect::<Vec<_>>();
        versions.sort_by_key(|v| &v.version);
        versions.last().expect("No known lts node versions")
    }

    /// Returns a lts version by name
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_lts<S: AsRef<str> + Debug>(&self, lts_name: S) -> Option<&VersionMetadata> {
        let lts_version = self.lts_versions.get(lts_name.as_ref())?;
        self.get_latest_for_major(*lts_version)
    }

    /// Returns any version that fulfills the given requirement
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_fulfilling(&self, req: &VersionReq) -> Option<&VersionMetadata> {
        let fulfilling_versions = self
            .sorted_versions
            .iter()
            .map(|v| (*v).into())
            .filter(|v| req.matches(v))
            .collect::<Vec<_>>();

        let version = fulfilling_versions.last()?.clone().into();
        self.versions.get(&version)
    }

    /// Returns the info for the given version
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get(&self, version: &Version) -> Option<&VersionMetadata> {
        self.versions.get(&version.clone().into())
    }

    /// Returns any version that fulfills the given requirement
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_latest_for_major(&self, major: u8) -> Option<&VersionMetadata> {
        let fulfilling_versions = self
            .sorted_versions
            .iter()
            .filter(|v| v.major == major)
            .collect::<Vec<_>>();

        let version = fulfilling_versions.last()?;
        self.versions.get(version)
    }

    /// Creates the list of sorted versions
    /// It needs to be calculated once after creating the struct
    fn create_sorted_versions(&mut self) {
        self.sorted_versions = self.versions.keys().cloned().collect::<Vec<_>>();
        self.sorted_versions.sort();
    }
}
