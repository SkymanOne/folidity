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
use walkdir::WalkDir;

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
        if out_path.exists() {
            for entry in WalkDir::new(out_path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let f_name = entry.file_name().to_string_lossy();
                let sec = entry.metadata()?.modified()?;

                if f_name.ends_with(".fol") && sec.elapsed()?.as_secs() < 86400 {
                    anyhow::bail!(
                        "Project with this name already exist in {}",
                        out_dir.to_str().unwrap()
                    );
                }
            }
        } else {
            create_dir(&out_dir).map(|_| anyhow::anyhow!("Cannot create project directory."))?;
        }

        let contract_content = include_bytes!("../../../../examples/counter/counter.fol");
        let readme_content = include_bytes!("../../../../examples/counter/README.md");

        let mut contract_file = File::create(Path::new(&out_dir).join("counter.fol"))?;
        contract_file.write_all(contract_content)?;

        let mut readme_file = File::create(Path::new(&out_dir).join("README.md"))?;
        readme_file.write_all(readme_content)?;

        Ok(())
    }
}
