use folidity_diagnostics::Report;
use folidity_semantics::{
    ContractDefinition,
    GlobalSymbol,
    Span,
    SymbolInfo,
};
use indexmap::IndexMap;
use num_traits::CheckedAdd;

use crate::ast::{
    Chunk,
    FuncInfo,
    Instruction,
};

#[derive(Debug, Clone)]
pub struct TealArtifacts {
    /// Teal approval program bytes.
    approval_bytes: Vec<u8>,
    /// Teal clear program bytes.
    clear_bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct TealEmitter<'a> {
    /// Nested definition of the contract.
    pub definition: &'a ContractDefinition,
    /// List of chunks that are emitted into the final build.
    chunks: Vec<Chunk>,
    /// Mapping from function symbol to its teal method signature.
    pub func_infos: IndexMap<SymbolInfo, FuncInfo>,
    /// Errors and warning caused during emit process.
    pub diagnostics: Vec<Report>,
    /// Index for scratch space variable.
    ///
    /// We use `u8` as there are only 256 cells available.
    pub scratch_index: u8,
}

impl<'a> TealEmitter<'a> {
    pub fn new(definition: &'a ContractDefinition) -> Self {
        Self {
            definition,
            chunks: vec![],
            func_infos: IndexMap::new(),
            diagnostics: vec![],
            scratch_index: 0,
        }
    }

    pub fn scratch_index_incr(&mut self) -> Result<u8, ()> {
        let i = self.scratch_index;
        self.scratch_index.checked_add(1).ok_or_else(|| {
            self.diagnostics.push(Report::emit_error(
                Span::default(),
                "Exceeded variable count".to_string(),
            ))
        })?;

        Ok(i)
    }
}
