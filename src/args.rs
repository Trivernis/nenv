use std::str::FromStr;

use clap::{Parser, Subcommand};
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
    pub version: Version,
}

#[derive(Clone, Debug, Parser)]
pub struct UseArgs {
    #[arg()]
    pub version: Version,
}

impl FromStr for Version {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.to_lowercase();

        let version = match &*input {
            "latest" => Self::Latest,
            "lts" => Self::LatestLts,
            _ => {
                if let Ok(req) = VersionReq::parse(s) {
                    Self::Req(req)
                } else {
                    Self::Lts(s.to_lowercase())
                }
            }
        };

        Ok(version)
    }
}

#[derive(Clone, Debug)]
pub enum Version {
    Latest,
    LatestLts,
    Lts(String),
    Req(VersionReq),
}
