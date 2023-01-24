use std::{ffi::OsString, str::FromStr};

use crate::{
    config::ConfigAccess,
    consts::{BIN_DIR, CACHE_DIR, VERSION_FILE_PATH},
    error::VersionError,
    mapper::Mapper,
    repository::{NodeVersion, Repository},
    utils::prompt,
    version_detection::{self, VersionDetector},
};
use crossterm::style::Stylize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use miette::{Context, IntoDiagnostic, Result};
use tokio::fs;

pub struct Nenv {
    config: ConfigAccess,
    repo: Repository,
    active_version: NodeVersion,
}

impl Nenv {
    #[tracing::instrument(level = "debug")]
    pub async fn init() -> Result<Self> {
        let config = ConfigAccess::load().await?;
        let repo = Repository::init(config.clone()).await?;
        let default_version = { config.get().await.node.default_version.to_owned() };
        let active_version = Self::get_active_version().await.unwrap_or(default_version);

        Ok(Self {
            config,
            repo,
            active_version,
        })
    }

    /// Installs the given node version.
    /// Prompts if that version already exists
    #[tracing::instrument(skip(self))]
    pub async fn install(&mut self, version: NodeVersion) -> Result<()> {
        Self::clear_version_cache().await?;

        if self.repo.is_installed(&version).await?
            && !prompt(
                false,
                format!(
                    "The version {} is already installed. Reinstall?",
                    version.to_string().bold()
                ),
            )
        {
            println!("Nothing changed.");
            Ok(())
        } else {
            self.repo.install_version(&version).await?;
            self.active_version = version.to_owned();
            self.get_mapper().await?.remap_additive().await?;

            println!("Installed {}", version.to_string().bold());
            Ok(())
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn uninstall(&mut self, version: NodeVersion) -> Result<()> {
        if prompt(
            false,
            format!(
                "Do you really want to uninstall node {}?",
                version.to_string().bold()
            ),
        ) {
            self.repo.uninstall(&version).await?;
            println!("Node {} has been removed.", version.to_string().bold())
        } else {
            println!("Nothing changed.");
        }

        Ok(())
    }

    /// Sets the system-wide default version
    #[tracing::instrument(skip(self))]
    pub async fn set_system_default(&mut self, version: NodeVersion) -> Result<()> {
        self.active_version = version.to_owned();

        if !self.repo.is_installed(&version).await? {
            if prompt(
                false,
                format!("The version {version} is not installed. Do you want to install it?"),
            ) {
                self.repo.install_version(&version).await?;
                self.config.get_mut().await.node.default_version = version.to_owned();
                self.get_mapper().await?.remap_additive().await?;
                println!("Now using {}", version.to_string().bold());
            }

            Ok(())
        } else {
            self.get_mapper().await?.remap_additive().await?;
            self.config.get_mut().await.node.default_version = version.to_owned();
            println!("Now using {}", version.to_string().bold());

            Ok(())
        }
    }

    /// Executes a given node executable for the currently active version
    #[tracing::instrument(skip(self))]
    pub async fn exec(&mut self, command: String, args: Vec<OsString>) -> Result<i32> {
        if !self.repo.is_installed(&self.active_version).await? {
            self.repo.install_version(&self.active_version).await?;
        }
        let exit_status = self.get_mapper().await?.exec(command, args).await?;

        Ok(exit_status.code().unwrap_or(0))
    }

    /// Clears the version cache and remaps all executables
    #[tracing::instrument(skip(self))]
    pub async fn remap(&mut self) -> Result<()> {
        self.get_mapper().await?.remap().await
    }

    /// Lists the currently installed versions
    #[tracing::instrument(skip(self))]
    pub async fn list_versions(&mut self) -> Result<()> {
        let versions = self.repo.installed_versions();
        let active_version = self
            .repo
            .lookup_remote_version(&self.active_version)
            .await?;
        let active_version = active_version.version.into();

        println!("{}", "Installed versions:".bold());

        for version in versions {
            let info = self
                .repo
                .all_versions()
                .await?
                .get(&version)
                .ok_or_else(|| VersionError::unknown_version(version.to_string()))?;
            let lts = info
                .lts
                .as_ref()
                .map(|l| format!(" ({})", l.to_owned().green()))
                .unwrap_or_default();

            if version == active_version {
                println!(" {}{} [current]", version.to_string().blue().bold(), lts)
            } else {
                println!(" {}{}", version.to_string().blue(), lts)
            }
        }

        Ok(())
    }

    /// Initializes nenv and prompts for a default version.
    #[tracing::instrument(skip(self))]
    pub async fn init_nenv(&mut self) -> Result<()> {
        let items = vec!["latest", "lts", "custom"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a default node version")
            .items(&items)
            .default(0)
            .interact()
            .into_diagnostic()?;

        let version = if items[selection] == "custom" {
            let version_string: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter a version number: ")
                .interact_text()
                .into_diagnostic()?;
            NodeVersion::from_str(&version_string).unwrap()
        } else {
            NodeVersion::from_str(items[selection]).unwrap()
        };

        self.repo.install_version(&version).await?;

        println!("{}", "Initialized!".green());
        println!(
            "{}\n  {}\n{}",
            "Make sure to add".bold(),
            BIN_DIR.to_string_lossy().yellow(),
            "to your PATH environment variables.".bold()
        );

        Ok(())
    }

    /// Clears the download cache
    #[tracing::instrument(skip(self))]
    pub async fn clear_cache(&self) -> Result<()> {
        fs::remove_dir_all(&*CACHE_DIR)
            .await
            .into_diagnostic()
            .context("Removing cache directory")?;
        fs::create_dir_all(&*CACHE_DIR)
            .await
            .into_diagnostic()
            .context("Creating cache directory")?;
        println!("Cleared download cache.");

        Ok(())
    }

    /// Persits all changes made that aren't written to the disk yet
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn persist(&self) -> Result<()> {
        self.config.save().await
    }

    #[tracing::instrument(level = "debug")]
    async fn get_active_version() -> Option<NodeVersion> {
        version_detection::ParallelDetector::detect_version()
            .await
            .ok()
            .and_then(|v| v)
    }

    #[tracing::instrument(level = "debug")]
    async fn clear_version_cache() -> Result<()> {
        if VERSION_FILE_PATH.exists() {
            fs::remove_file(&*VERSION_FILE_PATH)
                .await
                .into_diagnostic()?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_mapper(&mut self) -> Result<Mapper> {
        let node_path = self
            .repo
            .get_version_path(&self.active_version)?
            .ok_or_else(|| VersionError::not_installed(self.active_version.to_owned()))?;
        Ok(Mapper::new(node_path))
    }
}
