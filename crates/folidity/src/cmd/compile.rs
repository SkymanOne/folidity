use anyhow::{
    Context,
    Result,
};
use folidity_emitter::teal::{
    TealArtifacts,
    TealEmitter,
};
use folidity_parser::parse;
use folidity_semantics::ContractDefinition;
use folidity_verifier::SymbolicExecutor;
use std::{
    ffi::OsString,
    fs::{
        create_dir,
        File,
    },
    io::Write,
    path::PathBuf,
};
use yansi::Paint;

use clap::Args;

use super::{
    build_report,
    exec,
    read_contract,
};

/// Check the contract's code for errors
/// and validate model consistency using static analysis and symbolic execution.
#[derive(Args)]
pub struct CompileCommand {
    /// Contract's file name
    #[clap(value_parser)]
    contract: OsString,
    /// Skip formal verification stage.
    #[clap(short, long)]
    skip_verify: bool,
}

impl CompileCommand {
    pub fn run(&self) -> Result<()> {
        let contract_contents = read_contract(&self.contract)?;
        let parse_result = parse(&contract_contents);
        let file_name = self.contract.to_str().context("Invalid filename")?;
        match parse_result {
            Ok(tree) => {
                let contract =
                    exec::<_, _, ContractDefinition>(&tree, &contract_contents, file_name)?;

                exec::<_, _, SymbolicExecutor>(&contract, &contract_contents, file_name)?;
                println!(
                    "{}",
                    "Program model is consistent and has satisfiable constraints."
                        .green()
                        .bold()
                );

                let artifacts = exec::<_, TealArtifacts, TealEmitter>(
                    &contract,
                    &contract_contents,
                    file_name,
                )?;

                self.write_output(&artifacts)?;

                Ok(())
            }
            Err(errors) => {
                build_report(
                    &contract_contents,
                    &errors,
                    self.contract.to_str().expect("Valid path name."),
                );
                anyhow::bail!("Error during parsing")
            }
        }
    }

    fn write_output(&self, artifacts: &TealArtifacts) -> Result<()> {
        let mut current_path = PathBuf::from(&self.contract);
        current_path.pop();

        current_path.push("build");

        if !current_path.exists() {
            create_dir(&current_path)?;
        }

        let mut approval_path = current_path.clone();
        approval_path.push("approval.teal");

        let mut clear_path = current_path.clone();
        clear_path.push("clear.teal");

        let mut approval_file = File::create(&approval_path)?;
        approval_file.write_all(&artifacts.approval_bytes)?;

        let mut clear_file = File::create(&clear_path)?;
        clear_file.write_all(&artifacts.clear_bytes)?;

        println!("{}", "Successfully executed compilation!".bold().green());
        println!(
            "{}: {}",
            "Approval program".bold().cyan(),
            approval_path.to_str().unwrap()
        );
        println!(
            "{}: {}",
            "Clear program".bold().cyan(),
            clear_path.to_str().unwrap()
        );

        Ok(())
    }
}
