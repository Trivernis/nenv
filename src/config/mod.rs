use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use miette::Context;
use miette::{IntoDiagnostic, Result};

use tokio::fs;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::error::SerializeTomlError;
use crate::{
    consts::{CFG_DIR, CFG_FILE_PATH},
    error::ParseConfigError,
};

use self::value::Config;

mod value;
pub use value::*;

#[derive(Clone)]
pub struct ConfigAccess {
    dirty: Arc<AtomicBool>,
    config: Arc<RwLock<Config>>,
}

pub struct ModifyGuard<'a, T>(ConfigAccess, RwLockWriteGuard<'a, T>);

impl<'a, T> Deref for ModifyGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<'a, T> DerefMut for ModifyGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<'a, T> Drop for ModifyGuard<'a, T> {
    fn drop(&mut self) {
        self.0
            .dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl ConfigAccess {
    /// Loads the config file from the default config path
    pub async fn load() -> Result<Self> {
        if !CFG_FILE_PATH.exists() {
            if !CFG_DIR.exists() {
                fs::create_dir_all(&*CFG_DIR)
                    .await
                    .into_diagnostic()
                    .context("creating config dir")?;
            }
            let cfg = Config::default();
            let access = Self::new(cfg);
            access.save().await?;

            Ok(access)
        } else {
            let cfg_string = fs::read_to_string(&*CFG_FILE_PATH)
                .await
                .into_diagnostic()
                .context("reading config file")?;

            let cfg = toml::from_str(&cfg_string)
                .map_err(|e| ParseConfigError::new("config.toml", cfg_string, e))?;
            tracing::debug!("{cfg:?}");

            Ok(Self::new(cfg))
        }
    }

    pub async fn get(&self) -> RwLockReadGuard<Config> {
        if self.dirty.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.save().await.expect("Failed so save config");
        }
        self.config.read().await
    }

    pub async fn get_mut(&self) -> ModifyGuard<Config> {
        if self.dirty.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.save().await.expect("Failed so save config");
        }
        ModifyGuard(self.clone(), self.config.write().await)
    }

    fn new(config: Config) -> Self {
        Self {
            dirty: Arc::new(AtomicBool::new(false)),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn save(&self) -> Result<()> {
        fs::write(
            &*CFG_FILE_PATH,
            toml::to_string_pretty(&*self.config.read().await).map_err(SerializeTomlError::from)?,
        )
        .await
        .into_diagnostic()
        .context("writing config file")?;

        Ok(())
    }
}
