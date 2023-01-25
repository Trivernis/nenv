use std::{fs::File, io::Write};

use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::{
    consts::INSTALLED_VERSION_FILE,
    versioning::{SimpleVersion, VersionMetadata},
};
use miette::{Context, IntoDiagnostic, Result};

#[derive(Serialize, Deserialize, Default)]
pub struct InstalledVersions {
    ordered_versions: Vec<(SimpleVersion, VersionMetadata)>,
}

impl InstalledVersions {
    pub fn new(mut versions: Vec<(SimpleVersion, VersionMetadata)>) -> Self {
        versions.sort_by_key(|e| e.0);
        versions.dedup_by_key(|e| e.0);
        Self {
            ordered_versions: versions,
        }
    }

    /// Loads the local versions
    pub fn load() -> Result<Self> {
        let reader = File::open(&*INSTALLED_VERSION_FILE)
            .into_diagnostic()
            .context("Opening local versions file")?;
        let versions = bincode::deserialize_from(reader)
            .into_diagnostic()
            .context("Deserializing local versions")?;

        Ok(versions)
    }

    /// Saves the local versions
    pub fn save(&self) -> Result<()> {
        let mut file = File::create(&*INSTALLED_VERSION_FILE)
            .into_diagnostic()
            .context("Opening local versions file")?;
        bincode::serialize_into(&mut file, &self)
            .into_diagnostic()
            .context("Serializing local versions")?;
        file.flush()
            .into_diagnostic()
            .context("Flushing local versions to file")?;

        Ok(())
    }

    /// Inserts a new version. This requires reordering the list
    pub fn insert(&mut self, version: (SimpleVersion, VersionMetadata)) {
        self.ordered_versions.push(version);
        self.ordered_versions.sort_by_key(|e| e.0);
        self.ordered_versions.dedup_by_key(|e| e.0);
    }

    /// Removes a version. This keeps the order intact
    pub fn remove(&mut self, version: &SimpleVersion) {
        self.ordered_versions.retain(|(v, _)| v != version)
    }

    pub fn all(&self) -> Vec<&SimpleVersion> {
        self.ordered_versions.iter().map(|(v, _)| v).collect()
    }

    pub fn latest(&self) -> Option<&VersionMetadata> {
        self.ordered_versions.last().map(|(_, m)| m)
    }

    pub fn latest_lts(&self) -> Option<&VersionMetadata> {
        self.ordered_versions
            .iter()
            .filter(|(_, m)| m.lts.is_some())
            .last()
            .map(|(_, m)| m)
    }

    pub fn lts<S: AsRef<str>>(&self, lts: S) -> Option<&VersionMetadata> {
        self.ordered_versions
            .iter()
            .filter_map(|(v, m)| Some((v, m.lts.clone()?, m)))
            .filter(|(_, n, _)| n == lts.as_ref())
            .last()
            .map(|(_, _, m)| m)
    }

    pub fn fulfilling(&self, req: &VersionReq) -> Option<&VersionMetadata> {
        self.ordered_versions
            .iter()
            .filter(|(v, _)| req.matches(&v.to_owned().into()))
            .last()
            .map(|(_, m)| m)
    }
}

impl From<Vec<VersionMetadata>> for InstalledVersions {
    fn from(versions: Vec<VersionMetadata>) -> Self {
        let versions = versions
            .into_iter()
            .map(|v| (v.version.to_owned(), v))
            .collect::<Vec<_>>();

        Self::new(versions)
    }
}
