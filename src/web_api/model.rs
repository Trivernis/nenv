use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize};

/// Represents a single nodejs version info entry
/// as retrieved from nodejs.org
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VersionInfo {
    #[serde(deserialize_with = "deserialize_prefixed_version")]
    pub version: semver::Version,
    pub date: String,
    pub modules: Option<String>,

    #[serde(deserialize_with = "deserialize_false_as_none")]
    pub lts: Option<String>,
    pub security: bool,
    #[serde(flatten)]
    pub module_versions: ModuleVersions,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModuleVersions {
    pub v8: String,
    pub npm: Option<String>,
    pub uv: Option<String>,
    pub zlib: Option<String>,
    pub openssl: Option<String>,
}

fn deserialize_false_as_none<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(String::deserialize(deserializer).ok())
}

fn deserialize_prefixed_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<semver::Version, D::Error> {
    let version = String::deserialize(deserializer)?;
    let version = if let Some(v) = version.strip_prefix('v') {
        Cow::Borrowed(v)
    } else {
        Cow::Owned(version)
    };
    let version = semver::Version::parse(version.as_ref()).map_err(serde::de::Error::custom)?;

    Ok(version)
}
