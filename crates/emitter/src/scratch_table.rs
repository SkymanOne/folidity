use folidity_semantics::symtable::{
    Scope,
    VariableSym,
};
use indexmap::IndexMap;

use crate::teal::TealEmitter;

#[derive(Debug, Clone)]
pub struct ScratchVariable {
    /// position in the scratch table.
    pub index: u8,
    /// size of a variable.
    pub size: u64,
}

/// Table of values stores in the scratch space.
#[derive(Debug, Clone, Default)]
pub struct ScratchTable {
    vars: IndexMap<usize, ScratchVariable>,
}

impl ScratchTable {
    // pub fn add_scope(&mut self, scope: &Scope, emitter: &mut TealEmitter) {
    //     for (no, v) in &scope.vars {
    //         self.vars.insert(
    //             *no,
    //             ScratchVariable {
    //                 index: emitter.scratch_index,
    //                 ty: v.ty.clone(),
    //             },
    //         );
    //         emitter.scratch_index += 1;
    //     }
    // }

    /// Add variable to the virtual scratch table.
    pub fn add_var(&mut self, var_no: usize, size: u64, emitter: &mut TealEmitter) -> u8 {
        let index = emitter.scratch_index;
        self.vars.insert(var_no, ScratchVariable { index, size });
        emitter.scratch_index += 1;
        index
    }

    pub fn get_var(&self, no: usize) -> Option<&ScratchVariable> {
        self.vars.get(&no)
    }
}
