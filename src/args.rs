use std::ffi::OsString;

use crate::repository::NodeVersion;
use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Parser)]
#[clap(infer_subcommands = true)]
pub struct Args {
    /// Prints verbose logs
    #[arg(long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Returns the nenv version
    #[command(short_flag = 'v', aliases = &["--version"])]
    Version,

    /// Initializes nenv directories and installs a default node version
    #[command()]
    Init,

    /// Installs the given node version
    #[command()]
    Install(InstallArgs),

    /// Uninstalls the given node version
    #[command()]
    Uninstall(UninstallArgs),

    /// Sets the specified version as the global default
    #[command()]
    Default(DefaultArgs),

    /// Creates wrapper scripts for node binaries
    /// so they can be found in the path and are executed
    /// with the correct node version. This will delete
    /// all binary wrappers that don't apply to the active node version.
    #[command()]
    RemapBinaries,

    /// Lists all available versions
    #[command(name = "list-versions")]
    ListVersions,

    /// Executes the given version specific  node executable
    #[command()]
    Exec(ExecArgs),

    /// Clears the download cache
    #[command()]
    ClearCache,
}

#[derive(Clone, Debug, Parser)]
pub struct ExecArgs {
    /// The command to execute
    #[arg()]
    pub command: String,

    /// The arguments for the command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<OsString>,
}

#[derive(Clone, Debug, Parser)]
pub struct InstallArgs {
    /// the version to install
    pub version: NodeVersion,
}

#[derive(Clone, Debug, Parser)]
pub struct UninstallArgs {
    /// the version to install
    pub version: NodeVersion,
}

#[derive(Clone, Debug, Parser)]
pub struct DefaultArgs {
    /// The version to set as default
    pub version: NodeVersion,
}
