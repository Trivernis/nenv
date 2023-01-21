use std::ffi::OsString;

use consts::VERSION_FILE_PATH;
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
use tokio::fs;

pub async fn install_version(version: NodeVersion) -> Result<()> {
    if VERSION_FILE_PATH.exists() {
        fs::remove_file(&*VERSION_FILE_PATH).await?;
    }
    let repo = get_repository().await?;

    if repo.is_installed(&version)? {
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

pub async fn set_default_version(version: NodeVersion) -> Result<()> {
    let mut mapper = get_mapper().await?;

    if !mapper.repository().is_installed(&version)?
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

    mapper.set_default_version(&version).await?;
    println!("Now using {}", version.to_string().bold());

    Ok(())
}

#[inline]
pub async fn exec(command: String, args: Vec<OsString>) -> Result<i32> {
    let mapper = get_mapper().await?;
    let active_version = mapper.active_version();

    if !mapper.repository().is_installed(active_version)? {
        mapper.repository().install_version(&active_version).await?;
    }
    let exit_status = mapper.exec(command, args).await?;

    Ok(exit_status.code().unwrap_or(0))
}

pub async fn refresh() -> Result<()> {
    get_mapper().await?.remap().await?;
    fs::remove_file(&*VERSION_FILE_PATH).await?;

    Ok(())
}

async fn get_repository() -> Result<Repository> {
    Repository::init(Config::load().await?).await
}

async fn get_mapper() -> Result<Mapper> {
    Ok(Mapper::load(get_repository().await?).await)
}
