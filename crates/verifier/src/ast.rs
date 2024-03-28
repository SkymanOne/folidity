use folidity_semantics::{
    ast::Expression,
    GlobalSymbol,
    Span,
    SymbolInfo,
};
use z3::{
    ast::{
        Bool,
        Dynamic,
        Int,
        Real,
        String,
    },
    Context,
    Solver,
};
/// A declaration in the code AST.
#[derive(Debug, Clone)]
pub struct Declaration<'ctx> {
    /// Info about the declaration
    pub decl_sym: GlobalSymbol,
    /// Parent of the declaration.
    pub parent: Option<SymbolInfo>,
    /// Constraint block of the declaration.
    pub block: ConstraintBlock<'ctx>,
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
}

/// Block of constraints of to be verified.
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

pub struct Variable<'ctx> {
    /// Location of the expression
    pub loc: Span,
    /// Element of the expression.
    pub element: Dynamic<'ctx>,
}

/// A list of Z3 concrete expression.
/// Translated from the semantics AST?
pub enum Z3Expression<'ctx> {
    Variable(Z3UnaryExpression<usize>),

    // Literals
    Int(Z3UnaryExpression<Int<'ctx>>),
    Real(Z3UnaryExpression<Real<'ctx>>),
    Boolean(Z3UnaryExpression<Bool<'ctx>>),
    String(Z3UnaryExpression<String<'ctx>>),

    // Maths operations.
    Multiply(Z3BinaryExpression),
    Divide(Z3BinaryExpression),
    Modulo(Z3BinaryExpression),
    Add(Z3BinaryExpression),
    Subtract(Z3BinaryExpression),

    // Boolean relations.
    Equal(Z3BinaryExpression),
    NotEqual(Z3BinaryExpression),
    Greater(Z3BinaryExpression),
    Less(Z3BinaryExpression),
    GreaterEq(Z3BinaryExpression),
    LessEq(Z3BinaryExpression),
    In(Z3BinaryExpression),
    Not(Z3UnaryExpression<Box<Expression>>),

    // Boolean operations.
    Or(Z3BinaryExpression),
    And(Z3BinaryExpression),

    List(Z3UnaryExpression<Vec<Expression>>),
}

/// Represents unary style expression.
///
/// # Example
/// `false`
#[derive(Clone, Debug, PartialEq)]
pub struct Z3UnaryExpression<T> {
    /// Location of the expression
    pub loc: Span,
    /// Element of the expression.
    pub element: T,
}

/// Represents binary-style expression.
///
/// # Example
///
/// - `10 + 2`
/// - `a > b`
#[derive(Clone, Debug, PartialEq)]
pub struct Z3BinaryExpression {
    /// Location of the parent expression.
    pub loc: Span,
    /// Left expression.
    pub left: Box<Expression>,
    /// Right expression
    pub right: Box<Expression>,
}
