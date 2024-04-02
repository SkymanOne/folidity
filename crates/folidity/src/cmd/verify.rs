use anyhow::{
    Context,
    Result,
};
use folidity_parser::parse;
use folidity_semantics::ContractDefinition;
use folidity_verifier::SymbolicExecutor;
use std::ffi::OsString;
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
pub struct VerifyCommand {
    /// Contract's file name
    #[clap(value_parser)]
    contract: OsString,
}

impl VerifyCommand {
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
}
