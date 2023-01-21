use std::str::FromStr;

use clap::{Parser, Subcommand};
use nenv::repository::NodeVersion;
use semver::VersionReq;

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

    #[command()]
    Default,

    #[command(short_flag = 'v', aliases = &["--version"])]
    Version,
}

#[derive(Clone, Debug, Parser)]
pub struct InstallArgs {
    #[arg()]
    pub version: NodeVersion,
}

#[derive(Clone, Debug, Parser)]
pub struct UseArgs {
    #[arg()]
    pub version: NodeVersion,
}
