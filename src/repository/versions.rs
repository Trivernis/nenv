use std::{collections::HashMap, fmt::Display};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio::fs;

use crate::{consts::VERSION_FILE_PATH, error::SerializeBincodeError, web_api::VersionInfo};
use miette::{Context, IntoDiagnostic, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct Versions {
    lts_versions: HashMap<String, u16>,
    versions: HashMap<SimpleVersion, SimpleVersionInfo>,
    // as this field is not serialized
    // it needs to be calculated after serialization
    #[serde(skip)]
    sorted_versions: Vec<SimpleVersion>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize, Hash)]
pub struct SimpleVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u32,
}

impl From<semver::Version> for SimpleVersion {
    fn from(value: semver::Version) -> Self {
        Self {
            major: value.major as u16,
            minor: value.minor as u16,
            patch: value.patch as u32,
        }
    }
}

impl From<SimpleVersion> for semver::Version {
    fn from(value: SimpleVersion) -> Self {
        Self::new(value.major as u64, value.minor as u64, value.patch as u64)
    }
}

impl Display for SimpleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            major,
            minor,
            patch,
        } = self;
        write!(f, "{major}.{minor}.{patch}")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimpleVersionInfo {
    pub version: SimpleVersion,
    pub lts: Option<String>,
}

impl From<VersionInfo> for SimpleVersionInfo {
    fn from(value: VersionInfo) -> Self {
        Self {
            version: value.version.into(),
            lts: value.lts.lts(),
        }
    }
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
            .filter_map(|v| Some((v.lts.lts_ref()?.to_lowercase(), v.version.major as u16)))
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
    pub fn latest(&self) -> &SimpleVersionInfo {
        self.versions
            .get(self.sorted_versions.last().expect("No known node versions"))
            .unwrap()
    }

    /// Returns the latest node lts version
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn latest_lts(&self) -> &SimpleVersionInfo {
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
    pub fn get_lts<S: AsRef<str> + Debug>(&self, lts_name: S) -> Option<&SimpleVersionInfo> {
        let lts_version = self.lts_versions.get(lts_name.as_ref())?;
        self.get_latest_for_major(*lts_version)
    }

    /// Returns any version that fulfills the given requirement
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_fulfilling(&self, req: &VersionReq) -> Option<&SimpleVersionInfo> {
        let fulfilling_versions = self
            .sorted_versions
            .iter()
            .map(|v| (*v).into())
            .filter(|v| req.matches(v))
            .collect::<Vec<_>>();

        let version = fulfilling_versions.last()?.clone().into();
        self.versions.get(&version).into()
    }

    /// Returns the info for the given version
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get(&self, version: &Version) -> Option<&SimpleVersionInfo> {
        self.versions.get(&version.clone().into())
    }

    /// Returns any version that fulfills the given requirement
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_latest_for_major(&self, major: u16) -> Option<&SimpleVersionInfo> {
        let fulfilling_versions = self
            .sorted_versions
            .iter()
            .filter(|v| v.major == major)
            .collect::<Vec<_>>();

        let version = fulfilling_versions.last()?;
        self.versions.get(&version).into()
    }

    /// Creates the list of sorted versions
    /// It needs to be calculated once after creating the struct
    fn create_sorted_versions(&mut self) {
        self.sorted_versions = self.versions.keys().cloned().collect::<Vec<_>>();
        self.sorted_versions.sort();
    }
}
