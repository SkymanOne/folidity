use folidity_diagnostics::{
    Paint,
    Report,
};
use folidity_semantics::{
    ContractDefinition,
    GlobalSymbol,
    SymbolInfo,
};
use indexmap::IndexMap;
use z3::{
    ast::Dynamic,
    Context,
    Solver,
    Sort,
};

use crate::{
    ast::{
        Constraint,
        Declaration,
    },
    solver::verify_constraints,
    Diagnostics,
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

        for i in 0..contract.functions.len() {
            let mut constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let m = &contract.models[i];
            let Some(bounds) = &m.bounds else {
                continue;
            };
            for e in &bounds.exprs {
                match Constraint::from_expr(e, self.context, &mut diagnostics, self) {
                    Ok(c) => constraints.insert(c.binding_sym, c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            let decl = Declaration {
                constraints,
                decl_sym: GlobalSymbol::Model(SymbolInfo {
                    loc: bounds.loc.clone(),
                    i,
                }),
                parent: m.parent.clone(),
            };

            self.declarations.push(decl);
        }

        for i in 0..contract.states.len() {
            let mut constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let s = &contract.states[i];
            let Some(bounds) = &s.bounds else {
                continue;
            };
            for e in &bounds.exprs {
                match Constraint::from_expr(e, self.context, &mut diagnostics, self) {
                    Ok(c) => constraints.insert(c.binding_sym, c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            let decl = Declaration {
                constraints,
                decl_sym: GlobalSymbol::State(SymbolInfo {
                    loc: bounds.loc.clone(),
                    i,
                }),
                parent: s.from.clone().map(|x| x.0),
            };

            self.declarations.push(decl);
        }

        for i in 0..contract.functions.len() {
            let mut constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let f = &contract.functions[i];
            let Some(bounds) = &f.bounds else {
                continue;
            };
            for e in &bounds.exprs {
                match Constraint::from_expr(e, self.context, &mut diagnostics, self) {
                    Ok(c) => constraints.insert(c.binding_sym, c),
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
                    loc: bounds.loc.clone(),
                    i,
                }),
                parent: parent_bound,
            };

            self.declarations.push(decl);
        }

        if error {
            self.diagnostics.extend(diagnostics);
            return Err(());
        }
        Ok(())
    }

    /// Verify individual blocks of constraints for satisfiability.
    pub fn verify_individual_blocks(&mut self, contract: &ContractDefinition) -> Result<(), ()> {
        let mut diagnostics: Diagnostics = vec![];
        let mut error = false;

        for d in &self.declarations {
            if let Err(errs) = verify_constraints(
                d.constraints
                    .values()
                    .collect::<Vec<&Constraint>>()
                    .as_slice(),
                self.context,
            ) {
                let mut notes: Diagnostics = vec![];
                for (i, e) in errs.iter().enumerate() {
                    let c = d.constraints.get(e).expect("constraints exists");
                    notes.push(Report::ver_error(
                        c.loc.clone(),
                        format!(
                            "This is a constraint {}. It contradicts {:?}",
                            e.yellow(),
                            &remove_element(&errs, i).red()
                        ),
                    ))
                }

                notes.push(Report::ver_info(
                    "Consider rewriting logical bounds to satisfy all constraints.".to_string(),
                ));

                diagnostics.push(Report::ver_error_with_extra(
                    d.decl_sym.loc().clone(),
                    format!(
                        "{} has unsatisfiable constraints.",
                        symbol_name(&d.decl_sym, contract)
                    ),
                    notes,
                ));

                error = true;
            }
        }
        if error {
            self.diagnostics.extend(diagnostics);
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

fn remove_element<T: Clone>(arr: &[T], i: usize) -> Vec<T> {
    let (first_part, second_part) = arr.split_at(i);
    let mut result = first_part.to_vec();
    if i < second_part.len() {
        result.extend_from_slice(&second_part[1..]);
    }
    result
}

fn symbol_name(sym: &GlobalSymbol, contract: &ContractDefinition) -> String {
    match sym {
        GlobalSymbol::Struct(s) => format!("struct {}", contract.structs[s.i].name.name.cyan()),
        GlobalSymbol::Model(s) => format!("model {}", contract.structs[s.i].name.name.cyan()),
        GlobalSymbol::Enum(s) => format!("enum {}", contract.structs[s.i].name.name.cyan()),
        GlobalSymbol::State(s) => format!("state {}", contract.structs[s.i].name.name.cyan()),
        GlobalSymbol::Function(s) => format!("function {}", contract.structs[s.i].name.name.cyan()),
    }
}
