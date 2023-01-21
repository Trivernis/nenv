use std::str::FromStr;

use clap::{Parser, Subcommand};

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
            "lts" => Self::Lts,
            _ => Self::SemVer(SemVersion::from_str(s)?),
        };

        Ok(version)
    }
}

#[derive(Clone, Debug)]
pub enum Version {
    Latest,
    Lts,
    SemVer(SemVersion),
}

#[derive(Clone, Debug)]
pub struct SemVersion {
    pub major: u8,
    pub minor: Option<u8>,
    pub patch: Option<u16>,
}

impl FromStr for SemVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut major = s;
        let mut minor = None;
        let mut patch = None;

        if let Some((maj, rest)) = s.split_once('.') {
            major = maj;

            if let Some((min, pat)) = rest.split_once('.') {
                minor = Some(min.parse().map_err(|_| "minor is not a number")?);
                patch = Some(pat.parse().map_err(|_| "patch is not a number")?);
            } else {
                minor = Some(rest.parse().map_err(|_| "minor is not a number")?);
            }
        }

        Ok(Self {
            major: major.parse().map_err(|_| "major is not a number")?,
            minor,
            patch,
        })
    }
}
