use anyhow::Result;
use ariadne::{
    Color,
    Fmt,
};
use folidity_parser::parse;
use folidity_semantics::resolve_semantics;
use std::ffi::OsString;

use clap::Args;

use super::{
    build_report,
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
                let def = resolve_semantics(&tree);
                if def.diagnostics.is_empty() {
                    println!("{}", "Contract has no known errors".fg(Color::Green));
                    Ok(())
                } else {
                    build_report(
                        &contract_contents,
                        &def.diagnostics,
                        self.contract.to_str().unwrap(),
                    );
                    anyhow::bail!("Syntactical checking failed.")
                }
            }
            Err(errors) => {
                build_report(&contract_contents, &errors, self.contract.to_str().unwrap());
                anyhow::bail!("Error during parsing")
            }
        }
    }
}
