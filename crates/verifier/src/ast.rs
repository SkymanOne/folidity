use std::rc::Rc;

use folidity_diagnostics::Report;
use folidity_semantics::{
    ast::Expression,
    GlobalSymbol,
    Span,
    SymbolInfo,
};
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
    transformer::transform_expr,
};
/// A declaration in the code AST.
#[derive(Debug, Default)]
pub struct Declaration<'ctx> {
    /// Info about the declaration
    pub decl_sym: GlobalSymbol,
    /// Parent of the declaration.
    pub parent: Option<SymbolInfo>,
    /// Constraint block of the declaration.
    pub constraints: Vec<Constraint<'ctx>>,
}

impl<'ctx> Declaration<'ctx> {
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
            .map(|c| {
                Constraint {
                    loc: c.loc.clone(),
                    binding_sym: c.binding_sym,
                    expr: c.expr.translate(new_ctx).clone(),
                }
            })
            .collect()
    }

    /// Transform the list of ids of constraints into concrete boolean constants.
    pub fn constraint_consts<'a>(&self, ctx: &'a Context) -> Vec<Bool<'a>> {
        self.constraints
            .iter()
            .map(|c| c.sym_to_const(ctx))
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

    pub fn from_expr<'a>(
        expr: &Expression,
        ctx: &'a Context,
        diagnostics: &mut Vec<Report>,
        executor: &mut SymbolicExecutor<'ctx>,
    ) -> Result<Constraint<'a>, ()> {
        let resolve_e = transform_expr(expr, &ctx, diagnostics, executor)?;
        let Some(bool_expr) = resolve_e.element.as_bool() else {
            diagnostics.push(Report::ver_error(
                resolve_e.loc.clone(),
                String::from("Expression must be boolean."),
            ));
            return Err(());
        };
        let (binding_const, n) = executor.create_constant(&Sort::bool(&ctx), &ctx);

        let binding_expr = binding_const
            .as_bool()
            .expect("must be bool")
            .implies(&bool_expr);

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
