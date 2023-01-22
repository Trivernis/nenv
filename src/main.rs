use std::process;

use args::Args;
use clap::Parser;

use nenv::Nenv;

mod consts;
pub mod error;
pub mod mapper;
pub mod repository;
mod utils;
mod web_api;
use miette::Result;
use xkcd_unreachable::xkcd_unreachable;

mod args;
mod config;
mod nenv;
mod version_detection;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    miette::set_panic_hook();
    let args: Args = Args::parse();

    if let args::Command::Version = &args.command {
        print_version();
        return Ok(());
    }

    let mut nenv = get_nenv().await?;

    match args.command {
        args::Command::Install(v) => nenv.install(v.version).await,
        args::Command::Default(v) => nenv.set_system_default(v.version).await,
        args::Command::Exec(args) => {
            let exit_code = nenv.exec(args.command, args.args).await?;

            process::exit(exit_code);
        }
        args::Command::Refresh => nenv.refresh().await,
        args::Command::ListVersions => nenv.list_versions().await,
        _ => xkcd_unreachable!(),
    }?;

    nenv.persist().await?;

    Ok(())
}

fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

async fn get_nenv() -> Result<Nenv> {
    Nenv::init().await
}
