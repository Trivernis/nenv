use args::Args;
use clap::Parser;

use nenv::repository::NodeVersion;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    let args: Args = Args::parse();
    match args.commmand {
        args::Command::Install(v) => nenv::install_version(v.version).await.unwrap(),
        args::Command::Use(v) => nenv::use_version(v.version).await.unwrap(),
        args::Command::Default => todo!(),
        args::Command::Version => todo!(),
    };
}
