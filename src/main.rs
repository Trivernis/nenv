use std::process;

use args::Args;
use clap::Parser;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    let args: Args = Args::parse();
    match args.commmand {
        args::Command::Install(v) => nenv::install_version(v.version).await.unwrap(),
        args::Command::Use(v) => nenv::use_version(v.version).await.unwrap(),
        args::Command::Version => print_version(),
        args::Command::Exec(args) => {
            let exit_code = nenv::exec(args.command, args.args).await.unwrap();
            process::exit(exit_code);
        }
    };
}

fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
