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
mod teal;

#[cfg(test)]
mod tests;

impl<'a> Runner<ContractDefinition, TealArtifacts> for TealEmitter<'a> {
    fn run(source: &ContractDefinition) -> Result<TealArtifacts, CompilationError>
    where
        Self: std::marker::Sized,
    {
        let _emitter = TealEmitter::new(source);
        todo!()
    }
}

pub fn add_padding(chunks: &mut Vec<Chunk>) {
    chunks.insert(0, Chunk::new_empty(Instruction::Empty));
    chunks.push(Chunk::new_empty(Instruction::Empty));
}
