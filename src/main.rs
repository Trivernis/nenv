use std::process;

use args::Args;
use clap::Parser;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> miette::Result<()> {
    let args: Args = Args::parse();

    match args.commmand {
        args::Command::Version => Ok(print_version()),
        args::Command::Install(v) => nenv::install_version(v.version).await,
        args::Command::Default(v) => nenv::set_default_version(v.version).await,
        args::Command::Exec(args) => {
            let exit_code = nenv::exec(args.command, args.args).await?;

            process::exit(exit_code);
        }
        args::Command::Refresh => nenv::refresh().await,
    }?;

    Ok(())
}

fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
