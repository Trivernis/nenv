use serde::{Deserialize, Serialize};

use crate::{consts::NODE_DIST_URL, repository::NodeVersion};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Node execution related config
    pub node: NodeConfig,

    /// Configuration for how to download node versions
    pub download: DownloadConfig,
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
