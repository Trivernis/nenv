use std::process;

use args::Args;
use clap::Parser;
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
use miette::{IntoDiagnostic, Result};
use tokio::fs;

use crate::error::VersionError;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    miette::set_panic_hook();
    let args: Args = Args::parse();

    match args.commmand {
        args::Command::Version => {
            print_version();
            Ok(())
        }
        args::Command::Install(v) => install_version(v.version).await,
        args::Command::Default(v) => set_default_version(v.version).await,
        args::Command::Exec(args) => {
            let exit_code = exec(args.command, args.args).await?;

            process::exit(exit_code);
        }
        args::Command::Refresh => refresh().await,
        args::Command::ListVersions => list_versions().await,
    }?;

    Ok(())
}

fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

/// Installs a given node version
async fn install_version(version: NodeVersion) -> Result<()> {
    if VERSION_FILE_PATH.exists() {
        fs::remove_file(&*VERSION_FILE_PATH)
            .await
            .into_diagnostic()?;
    }
    let repo = get_repository().await?;

    if repo.is_installed(&version)?
        && !Confirm::new()
            .with_prompt(format!(
                "The version {} is already installed. Reinstall?",
                version.to_string().bold()
            ))
            .default(false)
            .interact()
            .unwrap()
    {
        return Ok(());
    }
    repo.install_version(&version).await?;
    println!("Installed {}", version.to_string().bold());

    Ok(())
}

/// Sets a default system wide node version
async fn set_default_version(version: NodeVersion) -> Result<()> {
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

/// Exectues a given command
#[inline]
async fn exec(command: String, args: Vec<OsString>) -> Result<i32> {
    let mapper = get_mapper().await?;
    let active_version = mapper.active_version();

    if !mapper.repository().is_installed(active_version)? {
        mapper.repository().install_version(active_version).await?;
    }
    let exit_status = mapper.exec(command, args).await?;

    Ok(exit_status.code().unwrap_or(0))
}

/// Refreshes the version cache and mapped binaries
async fn refresh() -> Result<()> {
    get_mapper().await?.remap().await?;
    fs::remove_file(&*VERSION_FILE_PATH)
        .await
        .into_diagnostic()?;
    println!("Remapped binaries and cleared version cache");

    Ok(())
}

/// Lists all available node versions
async fn list_versions() -> Result<()> {
    let mapper = get_mapper().await?;
    let versions = mapper.repository().installed_versions().await?;
    let active_version = mapper
        .repository()
        .lookup_version(mapper.active_version())?;

    println!("{}", "Installed versions:".bold());

    for version in versions {
        let info = mapper
            .repository()
            .all_versions()
            .get(&version)
            .ok_or_else(|| VersionError::unknown_version(version.to_string()))?;
        let lts = info
            .lts
            .as_ref()
            .map(|l| format!(" ({})", l.to_owned().green()))
            .unwrap_or_default();

        if version == active_version.version {
            println!(" {}{} [current]", version.to_string().blue().bold(), lts)
        } else {
            println!(" {}{}", version.to_string().blue(), lts)
        }
    }

    Ok(())
}

async fn get_repository() -> Result<Repository> {
    Repository::init(Config::load().await?).await
}

async fn get_mapper() -> Result<Mapper> {
    Ok(Mapper::load(get_repository().await?).await)
}
