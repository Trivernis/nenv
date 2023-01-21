use args::Args;
use clap::Parser;

use nenv::repository::NodeVersion;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    let args: Args = Args::parse();
    match args.commmand {
        args::Command::Install(v) => nenv::install_version(version_to_req(v.version))
            .await
            .unwrap(),
        args::Command::Use(_) => todo!(),
        args::Command::Default => todo!(),
        args::Command::Version => todo!(),
    };
}

fn version_to_req(version: args::Version) -> NodeVersion {
    match version {
        args::Version::Latest => NodeVersion::Latest,
        args::Version::LatestLts => NodeVersion::LatestLts,
        args::Version::Req(req) => NodeVersion::Req(req),
        args::Version::Lts(lts_name) => NodeVersion::Lts(lts_name),
    }
}
