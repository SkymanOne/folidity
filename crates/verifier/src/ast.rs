use folidity_semantics::{
    ast::Expression,
    GlobalSymbol,
    Span,
    SymbolInfo,
};
use z3::{
    ast::Bool,
    Context,
    Solver,
};

pub struct Declaration<'ctx> {
    /// Info about the declaration
    pub decl_sym: GlobalSymbol,
    /// Parent of the declaration.
    pub parent: Option<SymbolInfo>,
    /// Constraint block of the declaration.
    pub block: ConstraintBlock<'ctx>,
}

#[derive(Debug, Clone)]
pub struct Constraint<'ctx> {
    /// Location of the constraint in the original code.
    pub loc: Span,
    /// Binding constraint symbol id to track it across contexts.
    ///
    /// e.g. `k!0 => a > 10`
    /// where `0` is the id of the symbol.
    pub binding_sym: u32,
    /// Boolean expression.
    pub expr: Bool<'ctx>,
}

impl<'ctx> Constraint<'ctx> {
    /// Produce a boolean constant in the context from the constraint symbol.
    pub fn sym_to_const<'a>(&self, ctx: &'a Context) -> Bool<'a> {
        Bool::new_const(ctx, self.binding_sym)
    }
}

// impl build_

#[derive(Debug, Clone)]
pub struct ConstraintBlock<'ctx> {
    /// Solver which is scoped to the specific constraint block.
    pub solver: Solver<'ctx>,
    /// List of constraints in the given block.
    pub constraints: Vec<Constraint<'ctx>>,
}

impl<'ctx> ConstraintBlock<'ctx> {
    /// Translate context to the new solver.
    pub fn translate_to_solver<'a>(&self, solver: &Solver<'a>) -> Solver<'a> {
        let new_ctx = solver.get_context();
        self.solver.translate(new_ctx)
    }

    /// Transform the list of ids of constraints into concrete boolean constants.
    pub fn constraint_consts<'a>(&self, ctx: &'a Context) -> Vec<Bool<'a>> {
        self.constraints
            .iter()
            .map(|c| c.sym_to_const(ctx))
            .collect()
    }
}
