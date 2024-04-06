use folidity_semantics::{
    CompilationError,
    ContractDefinition,
    Runner,
};
use teal::{
    TealArtifacts,
    TealEmitter,
};

mod expression;
mod instruction;
mod scratch_table;
mod teal;

impl<'a> Runner<ContractDefinition, TealArtifacts> for TealEmitter<'a> {
    fn run(source: &ContractDefinition) -> Result<TealArtifacts, CompilationError>
    where
        Self: std::marker::Sized,
    {
        let _emitter = TealEmitter::new(source);
        todo!()
    }
}
