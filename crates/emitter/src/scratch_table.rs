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

    pub fn get_var_mut(&mut self, no: usize) -> Option<&mut ScratchVariable> {
        self.vars.get_mut(&no)
    }
}
