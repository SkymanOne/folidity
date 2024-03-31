use folidity_diagnostics::Report;
use folidity_semantics::{
    ContractDefinition,
    GlobalSymbol,
    SymbolInfo,
};
use z3::{
    ast::Dynamic,
    Context,
    Solver,
    Sort,
};

use crate::ast::{
    Constraint,
    Declaration,
};
#[derive(Debug)]
pub struct SymbolicExecutor<'ctx> {
    /// Global solver of the executor.
    ///
    /// We encapsulate it as it can't be easily transferred between scopes.
    context: &'ctx Context,
    /// List of resolved declaration to verify.
    pub declarations: Vec<Declaration<'ctx>>,
    /// Symbol counter to track boolean constants across the program.
    pub symbol_counter: u32,
    pub diagnostics: Vec<Report>,
}

impl<'ctx> SymbolicExecutor<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            declarations: vec![],
            diagnostics: vec![],
            symbol_counter: 0,
        }
    }

    /// Resolve Model, State and Function bounds into declarations with constraints.
    pub fn resolve_declarations(&mut self, contract: &ContractDefinition) -> Result<(), ()> {
        let mut error = false;
        let mut diagnostics = Vec::new();
        (0..contract.models.len()).for_each(|i| {
            let mut constraints: Vec<Constraint> = vec![];
            let m = &contract.models[i];
            for e in &m.bounds {
                match Constraint::from_expr(e, self.context, &mut diagnostics, self) {
                    Ok(c) => constraints.push(c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            let decl = Declaration {
                constraints,
                decl_sym: GlobalSymbol::Model(SymbolInfo {
                    loc: m.loc.clone(),
                    i,
                }),
                parent: m.parent.clone(),
            };

            self.declarations.push(decl);
        });

        (0..contract.states.len()).for_each(|i| {
            let mut constraints: Vec<Constraint> = vec![];
            let s = &contract.states[i];
            for e in &s.bounds {
                match Constraint::from_expr(e, self.context, &mut diagnostics, self) {
                    Ok(c) => constraints.push(c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            let decl = Declaration {
                constraints,
                decl_sym: GlobalSymbol::State(SymbolInfo {
                    loc: s.loc.clone(),
                    i,
                }),
                parent: s.from.clone().map(|x| x.0),
            };

            self.declarations.push(decl);
        });

        (0..contract.functions.len()).for_each(|i| {
            let mut constraints: Vec<Constraint> = vec![];
            let f = &contract.functions[i];
            for e in &f.bounds {
                match Constraint::from_expr(e, self.context, &mut diagnostics, self) {
                    Ok(c) => constraints.push(c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }

            let parent_bound = match f.state_bound.clone() {
                Some(b) => {
                    match b.from {
                        Some(f) => Some(f.ty),
                        None => None,
                    }
                }
                None => None,
            };
            let decl = Declaration {
                constraints,
                decl_sym: GlobalSymbol::Model(SymbolInfo {
                    loc: f.loc.clone(),
                    i,
                }),
                parent: parent_bound,
            };

            self.declarations.push(decl);
        });

        if error {
            return Err(());
        }
        Ok(())
    }

    /// Create a Z3 constant with the current symbol counter as a name while increasing
    /// the counter.
    pub fn create_constant(&mut self, sort: &Sort<'ctx>) -> (Dynamic<'ctx>, u32) {
        let id = self.symbol_counter;
        let c = Dynamic::new_const(self.context, id, sort);
        self.symbol_counter += 1;
        (c, id)
    }

    /// Retrieve the context of the internal `solver`.
    pub fn context(&self) -> &Context {
        self.context
    }

    pub fn transfer_context(&mut self, solver: Solver<'_>) -> Solver<'_> {
        solver.translate(self.context)
    }
}
