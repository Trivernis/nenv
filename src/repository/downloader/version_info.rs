use serde::{Deserialize, Deserializer};

/// Represents a single nodejs version info entry
/// as retrieved from nodejs.org
#[derive(Clone, Debug, Deserialize)]
pub struct VersionInfo {
    #[serde(deserialize_with = "deserialize_prefixed_version")]
    pub version: semver::Version,
    pub date: String,
    pub modules: Option<String>,

    pub lts: LtsInfo,
    pub security: bool,
    pub v8: String,
    pub npm: Option<String>,
    pub uv: Option<String>,
    pub zlib: Option<String>,
    pub openssl: Option<String>,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum LtsInfo {
    Version(String),
    NotLts(bool),
}

impl LtsInfo {
    pub fn lts(self) -> Option<String> {
        match self {
            LtsInfo::Version(v) => Some(v),
            LtsInfo::NotLts(_) => None,
        }
    }
    pub fn lts_ref(&self) -> Option<&String> {
        match &self {
            LtsInfo::Version(v) => Some(v),
            LtsInfo::NotLts(_) => None,
        }
    }
}

fn deserialize_prefixed_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<semver::Version, D::Error> {
    let version = String::deserialize(deserializer)?;
    let version = semver::Version::parse(version.trim_start_matches('v'))
        .map_err(serde::de::Error::custom)?;

    Ok(version)
}
