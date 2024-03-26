use anyhow::Result;
use folidity_parser::parse;
use folidity_semantics::ContractDefinition;
use std::ffi::OsString;

use clap::Args;

use super::{
    build_report,
    exec,
    read_contract,
};

/// Check the contract's code for parser, semantic and type errors.
#[derive(Args)]
pub struct CheckCommand {
    /// Contract's file name
    #[clap(value_parser)]
    contract: OsString,
}

impl CheckCommand {
    pub fn run(&self) -> Result<()> {
        let contract_contents = read_contract(&self.contract)?;
        let parse_result = parse(&contract_contents);
        match parse_result {
            Ok(tree) => {
                let _ = exec::<_, ContractDefinition>(
                    &tree,
                    &contract_contents,
                    self.contract.to_str().unwrap(),
                )?;
                Ok(())
            }
            Err(errors) => {
                build_report(&contract_contents, &errors, self.contract.to_str().unwrap());
                anyhow::bail!("Error during parsing")
            }
        }
    }
}
