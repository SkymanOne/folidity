use clap::Parser;
use cmd::Commands;

mod cmd;

#[derive(Parser)]
#[command(author = env!("CARGO_PKG_AUTHORS"), version = concat!("version ", env!("CARGO_PKG_VERSION")), about = env!("CARGO_PKG_DESCRIPTION"), subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    match cli.command.run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err:?}");
            std::process::exit(1);
        }
    }
}
