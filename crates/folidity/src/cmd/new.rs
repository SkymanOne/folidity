use anyhow::Result;
use std::{
    ffi::OsString,
    fs::{
        create_dir,
        File,
    },
    io::Write,
    path::Path,
};

use clap::Args;

/// Creates a new templated `folidity` counter project.
/// with a basic contract, README and approval teal code.
#[derive(Args)]
pub struct NewCommand {
    /// Path to the new project.
    /// If empty, the project will be created in the current dir.
    #[clap(value_parser)]
    name: Option<OsString>,
}

impl NewCommand {
    pub fn run(&self) -> Result<()> {
        let out_dir = self.name.clone().unwrap_or(OsString::from("."));
        let out_path = Path::new(&out_dir);
        if !out_path.exists() {
            create_dir(&out_dir)?;
        }

        let contract_path = out_path.join("contract.fol");
        if contract_path.exists() {
            anyhow::bail!(
                "Project with this name already exist in {}",
                out_dir.to_str().unwrap_or(".")
            );
        }

        let contract_content = include_bytes!("../../../../examples/counter/contract.fol");
        let readme_content = include_bytes!("../../../../examples/counter/README.md");

        let mut contract_file = File::create(contract_path)?;
        contract_file.write_all(contract_content)?;

        let mut readme_file = File::create(out_path.join("README.md"))?;
        readme_file.write_all(readme_content)?;

        Ok(())
    }
}
