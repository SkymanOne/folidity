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
use folidity_diagnostics::{
    Level,
    Report,
    Span,
};
use folidity_semantics::{
    CompilationError,
    Runner,
};
use yansi::Paint;

use self::{
    check::CheckCommand,
    new::NewCommand,
    verify::VerifyCommand,
};
use ariadne::{
    Color,
    Label,
    Report as PrettyReport,
    Source,
};

mod check;
mod new;
mod verify;

#[derive(Subcommand)]
pub enum Commands {
    New(NewCommand),
    Check(CheckCommand),
    Verify(VerifyCommand),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Commands::New(cmd) => cmd.run(),
            Commands::Check(cmd) => cmd.run(),
            Commands::Verify(cmd) => cmd.run(),
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
        let notes: Vec<Label<(&str, Span)>> = r
            .additional_info
            .iter()
            .filter(|x| x.level != Level::Info)
            .map(|ra| {
                Label::new((file_name, ra.loc.clone()))
                    .with_message(ra.message.clone())
                    .with_color(Color::Yellow)
            })
            .collect();
        let title = format!("{} detected.", r.error_type.cyan().underline(),);
        PrettyReport::build(r.level.clone().into(), file_name, r.loc.start)
            .with_message(title)
            .with_label(
                Label::new((file_name, r.loc.clone()))
                    .with_message(r.message.clone())
                    .with_color(Color::Yellow),
            )
            .with_labels(notes)
            .with_note(r.note.clone())
            .finish()
            .print((file_name, Source::from(content)))
            .unwrap();
    }
}

/// Execute the compilation stage using the runner.
pub fn exec<I, O, W: Runner<I, O>>(
    input: &I,
    contract_contents: &str,
    file_name: &str,
) -> Result<O> {
    W::run(input).map_err(|e| {
        let reports = e.diagnostics();
        build_report(contract_contents, reports, file_name);
        match e {
            CompilationError::Syntax(_) => anyhow::anyhow!("Syntactical error occurred"),
            CompilationError::Formal(_) => anyhow::anyhow!("Verification failed"),
            CompilationError::Emit(_) => anyhow::anyhow!("Compilation failed"),
        }
    })
}
