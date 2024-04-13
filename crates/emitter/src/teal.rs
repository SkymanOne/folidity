use folidity_diagnostics::Report;
use folidity_semantics::{
    ContractDefinition,
    GlobalSymbol,
    SymbolInfo,
};
use indexmap::IndexMap;

use crate::instruction::{
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
}
