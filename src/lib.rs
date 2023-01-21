use crossterm::style::Stylize;
use mapper::Mapper;
use repository::{config::Config, NodeVersion, Repository};

mod consts;
pub mod error;
pub mod mapper;
pub mod repository;
mod utils;
mod web_api;
use dialoguer::Confirm;
use error::Result;

pub async fn install_version(version: NodeVersion) -> Result<()> {
    let repo = get_repository().await?;

    if repo.is_installed(&version).await? {
        if !Confirm::new()
            .with_prompt("The version {version} is already installed. Reinstall?")
            .default(false)
            .interact()
            .unwrap()
        {
            return Ok(());
        }
    }
    repo.install_version(&version).await?;
    println!("Installed {}", version.to_string().bold());

    Ok(())
}

pub async fn use_version(version: NodeVersion) -> Result<()> {
    let mut mapper = get_mapper().await?;

    if !mapper.repository().is_installed(&version).await?
        && Confirm::new()
            .with_prompt(format!(
                "The version {version} is not installed. Do you want to install it?"
            ))
            .default(false)
            .interact()
            .unwrap()
    {
        mapper.repository().install_version(&version).await?;
    }

    mapper.use_version(&version).await?;
    println!("Now using {}", version.to_string().bold());

    Ok(())
}

async fn get_repository() -> Result<Repository> {
    Repository::init(Config::load().await?).await
}

async fn get_mapper() -> Result<Mapper> {
    Ok(Mapper::new(get_repository().await?))
}
