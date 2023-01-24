use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::repository::downloader::VersionInfo;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize, Hash)]
pub struct SimpleVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
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
            major: value.major as u8,
            minor: value.minor as u8,
            patch: value.patch as u16,
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
