use std::ffi::OsString;

use clap::{Parser, Subcommand};
use nenv::repository::NodeVersion;

#[derive(Clone, Debug, Parser)]
#[clap(infer_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub commmand: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command()]
    Install(InstallArgs),

    #[command()]
    Use(UseArgs),

    #[command(short_flag = 'v', aliases = &["--version"])]
    Version,

    #[command()]
    Exec(ExecArgs),
}

#[derive(Clone, Debug, Parser)]
pub struct ExecArgs {
    #[arg()]
    pub command: String,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<OsString>,
}

#[derive(Clone, Debug, Parser)]
pub struct InstallArgs {
    pub version: NodeVersion,
}

#[derive(Clone, Debug, Parser)]
pub struct UseArgs {
    #[arg()]
    pub version: NodeVersion,
}
