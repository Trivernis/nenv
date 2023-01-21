use std::collections::HashMap;

use semver::{Version, VersionReq};

use crate::web_api::VersionInfo;

pub struct Versions {
    lts_versions: HashMap<String, VersionReq>,
    versions: HashMap<Version, VersionInfo>,
}

impl Versions {
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
        let versions = all_versions
            .into_iter()
            .map(|v| (v.version.to_owned(), v))
            .collect::<HashMap<_, _>>();

        Self {
            lts_versions,
            versions,
        }
    }

    /// Returns the latest known node version
    pub fn latest(&self) -> &VersionInfo {
        let mut versions = self.versions.keys().collect::<Vec<_>>();
        versions.sort();

        self.versions
            .get(versions.last().expect("No known node versions"))
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
        let mut versions = self
            .versions
            .keys()
            .filter(|v| req.matches(v))
            .collect::<Vec<_>>();
        versions.sort();

        self.versions.get(versions.last()?)
    }
}
