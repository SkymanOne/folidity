use folidity_diagnostics::{
    Paint,
    Report,
};
use folidity_semantics::{
    ast::StateBody,
    ContractDefinition,
    DelayedDeclaration,
    GlobalSymbol,
    SymbolInfo,
};
use indexmap::IndexMap;
use z3::{
    ast::Dynamic,
    Context,
    Sort,
};

use crate::{
    ast::{
        Constraint,
        DeclarationBounds,
        Delays,
        Z3Scope,
    },
    solver::verify_constraints,
    transformer::type_to_sort,
    Diagnostics,
};
#[derive(Debug)]
pub struct SymbolicExecutor<'ctx> {
    /// Global solver of the executor.
    ///
    /// We encapsulate it as it can't be easily transferred between scopes.
    context: &'ctx Context,
    /// List of resolved declaration to verify.
    pub declarations: IndexMap<GlobalSymbol, DeclarationBounds<'ctx>>,
    /// Symbol counter to track boolean constants across the program.
    pub symbol_counter: u32,
    pub diagnostics: Vec<Report>,
}

impl<'ctx> SymbolicExecutor<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            declarations: IndexMap::new(),
            diagnostics: vec![],
            symbol_counter: 0,
        }
    }

    /// Resolve Model, State and Function declarations,
    /// and construct local z3 scopes of Z3 constants.
    ///
    /// Any declarations with links (e.g parents) are added to delay for a later resolve.
    ///
    /// We don't delay models as it is implies that any model inheritance introduces
    /// refinement to constraints. Therefore, its inherits the fields and provides its own
    /// constraints
    pub fn resolve_declarations<'a>(&mut self, contract: &'a ContractDefinition) -> Delays<'a> {
        let mut delays = Delays::default();

        for i in 0..contract.models.len() {
            let constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let m = &contract.models[i];
            let mut loc = m.loc.clone();
            let mut scope = Z3Scope::default();

            if let Some(bounds) = &m.bounds {
                loc = bounds.loc.clone();
            }
            let fields = m.fields(contract);
            for var in &fields {
                let _ = scope.create_or_get(
                    &var.name.name,
                    type_to_sort(&var.ty.ty, self.context),
                    self.context,
                    self,
                );
            }

            let decl = DeclarationBounds {
                constraints,
                loc,
                scope,
                links: vec![],
            };

            let sym = GlobalSymbol::Model(SymbolInfo {
                loc: m.loc.clone(),
                i,
            });

            self.declarations.insert(sym, decl);
        }

        for i in 0..contract.states.len() {
            let constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let current_index = self.declarations.len();
            let s = &contract.states[i];
            let mut loc = s.loc.clone();
            if let Some(bounds) = &s.bounds {
                loc = bounds.loc.clone();
            }

            let mut scope = Z3Scope::default();
            let mut add_delay = match &s.body {
                Some(StateBody::Raw(fields)) => {
                    for f in fields {
                        let _ = scope.create_or_get(
                            &f.name.name,
                            type_to_sort(&f.ty.ty, self.context),
                            self.context,
                            self,
                        );
                    }
                    false
                }
                Some(StateBody::Model(_)) => true,
                _ => false,
            };

            add_delay |= s.from.is_some();

            if add_delay {
                delays.state_delay.push(DelayedDeclaration {
                    decl: s,
                    i: current_index,
                });
            }

            let decl = DeclarationBounds {
                constraints,
                loc,
                scope,
                links: vec![],
            };

            let sym = GlobalSymbol::State(SymbolInfo {
                loc: s.loc.clone(),
                i,
            });
            self.declarations.insert(sym, decl);
        }

        for i in 0..contract.functions.len() {
            let constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let current_index = self.declarations.len();
            let f = &contract.functions[i];
            let mut loc = f.loc.clone();
            if let Some(bounds) = &f.bounds {
                loc = bounds.loc.clone();
            }

            let mut scope = Z3Scope::default();
            for (_, p) in &f.params {
                let _ = scope.create_or_get(
                    &f.name.name,
                    type_to_sort(&p.ty.ty, self.context),
                    self.context,
                    self,
                );
            }

            if f.state_bound.is_some() {
                delays.func_delay.push(DelayedDeclaration {
                    decl: f,
                    i: current_index,
                });
            }

            let decl = DeclarationBounds {
                constraints,
                loc,
                scope,
                links: vec![],
            };

            let sym = GlobalSymbol::Function(SymbolInfo {
                loc: f.loc.clone(),
                i,
            });
            self.declarations.insert(sym, decl);
        }

        delays
    }

    /// Resolve State and Function delay to finalise its scopes.
    pub fn resolve_links(&mut self, delays: Delays<'_>, contract: &ContractDefinition) {
        for s in &delays.state_delay {
            if let Some(StateBody::Model(model_sym)) = &s.decl.body {
                let mut links: Vec<GlobalSymbol> = vec![];
                let (m_sym, model_bound) = self
                    .declarations
                    .get_key_value(&GlobalSymbol::Model(model_sym.clone()))
                    .expect("Model should exist.");

                let m_scope = model_bound.scope.consts.clone();
                let mut m_sym = Some(m_sym.clone());
                while m_sym.is_some() {
                    let Some(sym) = &m_sym else {
                        break;
                    };
                    links.push(sym.clone());
                    let model_decl = &contract.models[sym.symbol_info().i];
                    m_sym = model_decl
                        .parent
                        .as_ref()
                        .map(|info| GlobalSymbol::Model(info.clone()))
                        .clone();
                }

                let s_bound = &mut self.declarations[s.i];
                s_bound.scope.consts.extend(m_scope);

                if let Some(from) = &s.decl.from {
                    links.push(GlobalSymbol::State(from.0.clone()))
                }
                s_bound.links = links;
            }
        }

        for m in &delays.func_delay {
            let f_bound = &mut self.declarations[m.i];
            let mut links: Vec<GlobalSymbol> = vec![];
            if let Some(sb) = &m.decl.state_bound {
                if let Some(from) = &sb.from {
                    links.push(GlobalSymbol::State(from.ty.clone()));
                }

                for t in &sb.to {
                    links.push(GlobalSymbol::State(t.ty.clone()));
                }
            }
            f_bound.links = links;
        }
    }

    pub fn resolve_bounds(&mut self, contract: &ContractDefinition) -> Result<(), ()> {
        let mut error = false;
        let mut diagnostics: Diagnostics = vec![];

        for (i, m) in contract.models.iter().enumerate() {
            let Some(bounds) = &m.bounds else {
                continue;
            };
            let mut constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let scope = &m.scope;
            let sym = GlobalSymbol::Model(SymbolInfo::new(m.loc.clone(), i));
            let mut z3_scope = Z3Scope::default();
            std::mem::swap(
                &mut z3_scope,
                &mut self.declarations.get_mut(&sym).expect("should exist").scope,
            );
            for e in &bounds.exprs {
                match Constraint::from_expr(
                    e,
                    self.context,
                    &mut z3_scope,
                    scope,
                    contract,
                    &mut diagnostics,
                    self,
                ) {
                    Ok(c) => constraints.insert(c.binding_sym, c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            std::mem::swap(
                &mut z3_scope,
                &mut self.declarations.get_mut(&sym).expect("should exist").scope,
            );
            self.declarations
                .get_mut(&sym)
                .expect("should exist")
                .constraints = constraints;
        }

        for (i, s) in contract.states.iter().enumerate() {
            let Some(bounds) = &s.bounds else {
                continue;
            };
            let mut constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let scope = &s.scope;
            let sym = GlobalSymbol::State(SymbolInfo::new(s.loc.clone(), i));
            let mut z3_scope = Z3Scope::default();
            std::mem::swap(
                &mut z3_scope,
                &mut self.declarations.get_mut(&sym).expect("should exist").scope,
            );
            for e in &bounds.exprs {
                match Constraint::from_expr(
                    e,
                    self.context,
                    &mut z3_scope,
                    scope,
                    contract,
                    &mut diagnostics,
                    self,
                ) {
                    Ok(c) => constraints.insert(c.binding_sym, c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            std::mem::swap(
                &mut z3_scope,
                &mut self.declarations.get_mut(&sym).expect("should exist").scope,
            );
            self.declarations
                .get_mut(&sym)
                .expect("should exist")
                .constraints = constraints;
        }

        for (i, f) in contract.functions.iter().enumerate() {
            let Some(bounds) = &f.bounds else {
                continue;
            };
            let mut constraints: IndexMap<u32, Constraint> = IndexMap::new();
            let scope = &f.scope;
            let sym = GlobalSymbol::Function(SymbolInfo::new(f.loc.clone(), i));
            let mut z3_scope = Z3Scope::default();
            std::mem::swap(
                &mut z3_scope,
                &mut self.declarations.get_mut(&sym).expect("should exist").scope,
            );
            for e in &bounds.exprs {
                match Constraint::from_expr(
                    e,
                    self.context,
                    &mut z3_scope,
                    scope,
                    contract,
                    &mut diagnostics,
                    self,
                ) {
                    Ok(c) => constraints.insert(c.binding_sym, c),
                    Err(_) => {
                        error = true;
                        continue;
                    }
                };
            }
            std::mem::swap(
                &mut z3_scope,
                &mut self.declarations.get_mut(&sym).expect("should exist").scope,
            );
            self.declarations
                .get_mut(&sym)
                .expect("should exist")
                .constraints = constraints;
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

        for (sym, d) in &self.declarations {
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
                    d.loc.clone(),
                    format!(
                        "{} has unsatisfiable constraints.",
                        symbol_name(sym, contract)
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
        GlobalSymbol::Model(s) => format!("model {}", contract.models[s.i].name.name.cyan()),
        GlobalSymbol::Enum(s) => format!("enum {}", contract.enums[s.i].name.name.cyan()),
        GlobalSymbol::State(s) => format!("state {}", contract.states[s.i].name.name.cyan()),
        GlobalSymbol::Function(s) => {
            format!("function {}", contract.functions[s.i].name.name.cyan())
        }
    }
}
