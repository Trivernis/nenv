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

    /// Installs the given node version
    #[command()]
    Install(InstallArgs),

    /// Sets the specified version as the global default
    #[command()]
    Default(DefaultArgs),

    /// Refreshes the node environment mappings and cache.
    /// This will erase all binary mappings not relevant to the current node version.
    #[command()]
    Refresh,

    /// Lists all available versions
    #[command(name = "list-versions")]
    ListVersions,

    /// Executes the given version specific  node executable
    #[command()]
    Exec(ExecArgs),

    /// Initializes nenv directories and installs a default node version
    #[command()]
    Init,
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
pub struct DefaultArgs {
    /// The version to set as default
    pub version: NodeVersion,
}
