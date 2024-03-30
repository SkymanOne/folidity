use folidity_diagnostics::Report;
use z3::{
    ast::Dynamic,
    Context,
    Solver,
    Sort,
};

use crate::ast::Declaration;

#[derive(Debug, Clone)]
pub struct SymbolicExecutor<'ctx> {
    /// Global solver of the executor.
    solver: Solver<'ctx>,
    /// List of resolved declaration to verify.
    pub declarations: Vec<Declaration<'ctx>>,
    /// Symbol counter to track boolean constants across the program.
    pub symbol_counter: u32,
    pub diagnostics: Vec<Report>,
}

impl<'ctx> SymbolicExecutor<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            solver: Solver::new(context),
            declarations: vec![],
            diagnostics: vec![],
            symbol_counter: 0,
        }
    }

    /// Create a Z3 constant with the current symbol counter as a name while increasing
    /// the counter.
    pub fn create_constant(
        &mut self,
        sort: &Sort<'ctx>,
        context: &'ctx Context,
    ) -> (Dynamic<'ctx>, u32) {
        let id = self.symbol_counter;
        let c = Dynamic::new_const(context, id, sort);
        self.symbol_counter += 1;
        (c, id)
    }
}
