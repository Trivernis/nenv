use repository::{config::Config, NodeVersion, Repository};

mod consts;
pub mod error;
pub mod repository;
mod web_api;
use error::Result;

pub async fn install_version(version: NodeVersion) -> Result<()> {
    get_repository().await?.install_version(version).await
}

async fn get_repository() -> Result<Repository> {
    Repository::init(Config::default()).await
}
