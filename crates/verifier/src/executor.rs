use folidity_diagnostics::Report;
use z3::{
    Context,
    Solver,
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
}
