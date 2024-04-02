use folidity_diagnostics::Report;
use folidity_semantics::{
    ast::{
        Expression,
        Function,
        StateDeclaration,
    },
    DelayedDeclaration,
    GlobalSymbol,
    Span,
};
use indexmap::IndexMap;
use z3::{
    ast::{
        Ast,
        Bool,
        Dynamic,
    },
    Context,
    Solver,
    Sort,
};

use crate::{
    executor::SymbolicExecutor,
    transformer::{
        create_constraint_const,
        transform_expr,
        TransformParams,
    },
};

#[derive(Debug, Default, Clone)]
pub struct Z3Scope {
    pub consts: IndexMap<String, u32>,
}

impl Z3Scope {
    pub fn create_or_get<'ctx>(
        &mut self,
        ident: &str,
        sort: Sort<'ctx>,
        ctx: &'ctx Context,
        executor: &mut SymbolicExecutor<'ctx>,
    ) -> Dynamic<'ctx> {
        if let Some(i) = self.consts.get(ident) {
            Dynamic::new_const(ctx, *i, &sort)
        } else {
            let (c, i) = executor.create_constant(&sort);
            self.consts.insert(ident.to_string(), i);
            c
        }
    }

    pub fn get<'ctx>(
        &self,
        ident: &str,
        sort: Sort<'ctx>,
        ctx: &'ctx Context,
    ) -> Option<Dynamic<'ctx>> {
        self.consts
            .get(ident)
            .map(|i| Dynamic::new_const(ctx, *i, &sort))
    }
}

#[derive(Debug, Default)]
pub struct Delays<'a> {
    pub state_delay: Vec<DelayedDeclaration<&'a StateDeclaration>>,
    pub func_delay: Vec<DelayedDeclaration<&'a Function>>,
}

/// A declaration in the code AST.
#[derive(Debug, Default)]
pub struct DeclarationBounds<'ctx> {
    /// `st` location block.
    pub loc: Span,
    /// Links of others declaration.
    pub links: Vec<GlobalSymbol>,
    /// Constraint block of the declaration.
    pub constraints: IndexMap<u32, Constraint<'ctx>>,
    /// Scope of the local constraints.
    pub scope: Z3Scope,
}

impl<'ctx> DeclarationBounds<'ctx> {
    /// Translate context to the new solver.
    pub fn translate_to_solver<'src_ctx>(
        &self,
        solver: &Solver<'src_ctx>,
    ) -> Vec<Constraint<'src_ctx>>
    where
        'ctx: 'src_ctx,
    {
        let new_ctx = solver.get_context();
        self.constraints
            .iter()
            .map(|(n, c)| {
                Constraint {
                    loc: c.loc.clone(),
                    binding_sym: *n,
                    expr: c.expr.translate(new_ctx).clone(),
                }
            })
            .collect()
    }

    /// Transform the list of ids of constraints into concrete boolean constants.
    pub fn constraint_consts<'a>(&self, ctx: &'a Context) -> Vec<Bool<'a>> {
        self.constraints
            .iter()
            .map(|(_, c)| c.sym_to_const(ctx))
            .collect()
    }
}

/// A singular constraint.
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

    pub fn from_expr(
        expr: &Expression,
        params: &mut TransformParams<'ctx, '_>,
    ) -> Result<Constraint<'ctx>, ()> {
        let resolve_e = transform_expr(expr, params)?;
        let Some(bool_expr) = resolve_e.element.as_bool() else {
            params.diagnostics.push(Report::ver_error(
                resolve_e.loc.clone(),
                String::from("Expression must be boolean."),
            ));
            return Err(());
        };
        let (binding_const, n) = create_constraint_const(params.ctx, params.executor);

        // create a binding boolean constant: `c => expr`, to track each constraint.
        let binding_expr = binding_const.implies(&bool_expr);

        Ok(Constraint {
            loc: resolve_e.loc.clone(),
            binding_sym: n,
            expr: binding_expr,
        })
    }
}

/// Represents unary style expression.
///
/// # Example
/// `false`
#[derive(Clone, Debug, PartialEq)]
pub struct Z3Expression<'ctx> {
    /// Location of the expression
    pub loc: Span,
    /// Element of the expression.
    pub element: Dynamic<'ctx>,
}

impl<'ctx> Z3Expression<'ctx> {
    pub fn new(loc: &Span, e: &impl Ast<'ctx>) -> Self {
        Self {
            loc: loc.clone(),
            element: Dynamic::from_ast(e),
        }
    }
}
