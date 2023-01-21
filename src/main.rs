use args::Args;
use clap::Parser;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    let args: Args = Args::parse();
    dbg!(args);
}
