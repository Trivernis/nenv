use std::{env, process};

use args::{Args, PinArgs, UnpinArgs};
use clap::Parser;

use nenv::Nenv;

mod consts;
pub mod error;
pub mod mapper;
pub mod repository;
mod utils;
use miette::Result;
use repository::NodeVersion;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use xkcd_unreachable::xkcd_unreachable;

mod args;
mod config;
mod nenv;
mod version_detection;
mod versioning;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    miette::set_panic_hook();
    let args: Args = Args::parse();

    if args.verbose {
        init_tracing();
    }

    if let args::Command::Version = &args.command {
        print_version();
        return Ok(());
    }

    let mut nenv = get_nenv(args.use_version.clone()).await?;

    match args.command {
        args::Command::Install(v) => nenv.install(v.version).await,
        args::Command::Uninstall(v) => nenv.uninstall(v.version).await,
        args::Command::SetDefault(v) => nenv.set_system_default(v.version).await,
        args::Command::Exec(args) => {
            let exit_code = nenv.exec(args.command, args.args).await?;

            process::exit(exit_code);
        }
        args::Command::RemapBinaries => nenv.remap().await,
        args::Command::ListVersions => nenv.list_versions().await,
        args::Command::Init => nenv.init_nenv().await,
        args::Command::ClearCache => nenv.clear_cache().await,
        args::Command::Pin(PinArgs { command, version }) => {
            nenv.pin_command(command, version).await
        }
        args::Command::Unpin(UnpinArgs { command }) => nenv.unpin_command(command).await,
        _ => xkcd_unreachable!(),
    }?;

    nenv.persist().await?;

    Ok(())
}

fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

async fn get_nenv(version_override: Option<NodeVersion>) -> Result<Nenv> {
    Nenv::init(version_override).await
}

fn init_tracing() {
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(LevelFilter::DEBUG)
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .compact()
        .init();
}
