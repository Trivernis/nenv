use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{consts::NODE_DIST_URL, repository::NodeVersion};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Node execution related config
    pub node: NodeConfig,

    /// Configuration for how to download node versions
    pub download: DownloadConfig,

    /// List of executables that are hardwired to a given node version
    /// and can still be executed from other versions with this given version.
    pub bins: HashMap<String, ExecutableConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeConfig {
    /// The default version if no version is specified
    /// in the `package.json` file or `NODE_VERSION` environment variable
    pub default_version: NodeVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadConfig {
    pub dist_base_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecutableConfig {
    /// The node version to run this executable with.
    /// This means that whatever the currently active version is
    /// the given executable will always be executed with the configured one.
    pub node_version: NodeVersion,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            default_version: NodeVersion::LatestLts,
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            dist_base_url: String::from(NODE_DIST_URL),
        }
    }
}
