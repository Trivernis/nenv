use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::repository::downloader::VersionInfo;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize, Hash)]
pub struct SimpleVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    /// The semver version
    pub version: SimpleVersion,
    /// The lts name of this version if it is an lts version
    pub lts: Option<String>,
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

impl From<VersionInfo> for VersionMetadata {
    fn from(value: VersionInfo) -> Self {
        Self {
            version: value.version.into(),
            lts: value.lts.lts(),
        }
    }
}
