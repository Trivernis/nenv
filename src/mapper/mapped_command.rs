use std::{
    ffi::OsString,
    path::PathBuf,
    process::{ExitStatus, Stdio},
};

use thiserror::Error;
use tokio::{io, process::Command};

pub struct MappedCommand {
    path: PathBuf,
    args: Vec<OsString>,
}

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("The command {0:?} could not be found")]
    NotFound(PathBuf),
}

impl MappedCommand {
    pub fn new(path: PathBuf, args: Vec<OsString>) -> Self {
        Self { path, args }
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn run(mut self) -> CommandResult<ExitStatus> {
        self.adjust_path()?;
        let exit_status = Command::new(self.path)
            .args(self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()
            .await?;

        Ok(exit_status)
    }

    #[cfg(not(target_os = "windows"))]
    fn adjust_path(&mut self) -> CommandResult<()> {
        if !self.path.exists() {
            Err(CommandError::NotFound(self.path.to_owned()))
        } else {
            Ok(())
        }
    }

    #[cfg(target_os = "windows")]
    fn adjust_path(&mut self) -> CommandResult<()> {
        if !self.path.exists() {
            let extensions = ["exe", "bat", "cmd", "ps1"];
            for extension in &extensions {
                let joined_path = self.path.with_extension(extension);

                if joined_path.exists() {
                    self.path = joined_path;
                    return Ok(());
                }
            }
            return Err(CommandError::NotFound(self.path.to_owned()));
        }

        Ok(())
    }
}
