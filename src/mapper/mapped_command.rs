use std::{
    ffi::OsString,
    io::{stderr, stdin, stdout},
    os::fd::{AsRawFd, FromRawFd},
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
    pub async fn run(self) -> CommandResult<ExitStatus> {
        if !self.path.exists() {
            return Err(CommandError::NotFound(self.path));
        }
        let (stdin, stdout, stderr) = unsafe {
            (
                Stdio::from_raw_fd(stdin().as_raw_fd()),
                Stdio::from_raw_fd(stdout().as_raw_fd()),
                Stdio::from_raw_fd(stderr().as_raw_fd()),
            )
        };
        let exit_status = Command::new(self.path)
            .args(self.args)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?
            .wait()
            .await?;

        Ok(exit_status)
    }
}
