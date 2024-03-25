use std::{
    ffi::OsString,
    fs::File,
    io::Read,
    path::Path,
};

use anyhow::{
    Context,
    Result,
};
use clap::Subcommand;
use folidity_diagnostics::Report;
use yansi::Paint;

use self::{
    check::CheckCommand,
    new::NewCommand,
};
use ariadne::{
    Color,
    Label,
    Report as PrettyReport,
    Source,
};

mod check;
mod new;

#[derive(Subcommand)]
pub enum Commands {
    New(NewCommand),
    Check(CheckCommand),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Commands::New(cmd) => cmd.run(),
            Commands::Check(cmd) => cmd.run(),
        }
    }
}

pub fn read_contract(path_str: &OsString) -> Result<String> {
    let path = Path::new(path_str);
    if !path.exists() {
        anyhow::bail!("File does not exist.");
    }
    let s = path.file_name().context("This is not a valid path.")?;
    if !s.to_string_lossy().ends_with(".fol") {
        anyhow::bail!("File is not a valid folidity contract.")
    }
    let mut file = File::open(path).context("Could not open a file.")?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .context("Failed to read file contents")?;
    Ok(buffer)
}

pub fn build_report(content: &str, diagnostics: &[Report], file_name: &str) {
    for r in diagnostics {
        PrettyReport::build(r.level.clone().into(), file_name, r.loc.start)
            .with_message(format!("{} detected.", r.error_type.cyan()))
            .with_label(
                Label::new((file_name, r.loc.clone()))
                    .with_message(r.message.clone())
                    .with_color(Color::Yellow),
            )
            .finish()
            .print((file_name, Source::from(content)))
            .unwrap();
    }
}
