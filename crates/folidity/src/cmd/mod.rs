use anyhow::Result;
use clap::Subcommand;

use self::new::NewCommand;

mod new;

#[derive(Subcommand)]
pub enum Commands {
    New(NewCommand),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Commands::New(cmd) => cmd.run(),
        }
    }
}
