use std::{
    ffi::OsString,
    path::PathBuf,
    process::{ExitStatus, Stdio},
};

use crate::error::CommandNotFoundError;
use miette::{Context, IntoDiagnostic, Result};
use tokio::process::Command;

pub struct MappedCommand {
    name: String,
    path: PathBuf,
    args: Vec<OsString>,
}

impl MappedCommand {
    pub fn new(name: String, path: PathBuf, args: Vec<OsString>) -> Self {
        Self { name, path, args }
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn run(mut self) -> Result<ExitStatus> {
        self.adjust_path()?;
        let exit_status = Command::new(self.path)
            .args(self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .into_diagnostic()
            .context("Running mapped command")?
            .wait()
            .await
            .into_diagnostic()
            .context("Waiting for command to exit")?;

        Ok(exit_status)
    }

    #[cfg(not(target_os = "windows"))]
    fn adjust_path(&mut self) -> Result<()> {
        if !self.path.exists() {
            Err(CommandNotFoundError::new(
                self.name.to_owned(),
                self.args.to_owned(),
                self.path.to_owned(),
            )
            .into())
        } else {
            Ok(())
        }
    }

    #[cfg(target_os = "windows")]
    fn adjust_path(&mut self) -> Result<()> {
        let extensions = ["exe", "bat", "cmd", "ps1"];
        for extension in &extensions {
            let joined_path = self.path.with_extension(extension);

            if joined_path.exists() {
                self.path = joined_path;
                return Ok(());
            }
        }
        Err(CommandNotFoundError::new(
            self.name.to_owned(),
            self.args.to_owned(),
            self.path.to_owned(),
        )
        .into())
    }
}
