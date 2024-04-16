use ast::{
    Chunk,
    Instruction,
};
use folidity_semantics::{
    CompilationError,
    ContractDefinition,
    Runner,
};
use teal::{
    TealArtifacts,
    TealEmitter,
};

mod ast;
mod expression;
mod function;
mod scratch_table;
mod statement;
pub mod teal;

#[cfg(test)]
mod tests;

impl<'a> Runner<ContractDefinition, TealArtifacts> for TealEmitter<'a> {
    fn run(source: &ContractDefinition) -> Result<TealArtifacts, CompilationError>
    where
        Self: std::marker::Sized,
    {
        let mut emitter = TealEmitter::new(source);
        emitter.emit_entry_point();
        if !emitter.emit_functions() {
            return Err(CompilationError::Emit(emitter.diagnostics));
        }

        let artifacts = emitter.compile();

        Ok(artifacts)
    }
}

pub fn add_padding(chunks: &mut Vec<Chunk>) {
    chunks.insert(0, Chunk::new_empty(Instruction::Empty));
    chunks.push(Chunk::new_empty(Instruction::Empty));
}
