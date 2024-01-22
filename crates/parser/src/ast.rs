use super::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub expressions: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    /// Location of the identifier.
    pub loc: Span,
    /// The name of the identifier.
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    FuncDeclaration(Box<FunctionDeclaration>)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Uint,
    Float,
    Char,
    String,
    Hex,
    Hash,
    Address,
    Unit,
    Bool,
    //todo: list types
}

/// Parameter declaration of the state.
/// `<ident> <ident>?`
#[derive(Clone, Debug, PartialEq)]
pub struct StateParam {
    pub loc: Span,
    /// State type identifier.
    pub ty: Identifier,
    /// Variable name identifier.
    pub name: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub loc: Span,
    /// Type identifier.
    pub ty: Type,
    /// Variable name identifier.
    pub name: Identifier,
}

/// View state modifier.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewState {
    pub loc: Span,
    pub param: StateParam,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum FunctionVisibility {
    Pub,
    View(ViewState),
    #[default]
    Priv,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FuncReturnType {
    Type(Type),
    ParamType(Param)
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateBound {
    pub loc: Span,
    /// Original state
    pub from: StateParam,
    /// Final state
    pub to: StateParam
}
/// Type alias for a list of function parameters.
pub type ParameterList = Vec<(Span, Option<Param>)>;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration {
    /// Location span of the function.
    pub loc: Span,
    /// Visibility of the function.
    pub vis: FunctionVisibility,
    /// Function return type declaration.
    pub return_ty: FuncReturnType,
    /// List of parameters.
    pub params: ParameterList,
    /// Bounds for the state transition.
    pub state_bound: StateBound,
    /// Function logical bounds
    pub st_block: StBlock,
    pub body: Statement
}

#[derive(Clone, Debug, PartialEq)]
pub struct StBlock {
    pub loc: Span,
    /// List of logic expressions
    pub exprs: Vec<Expression>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {

}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Number(UnaryExpression<String>),
    String(UnaryExpression<String>),
    Hex(UnaryExpression<String>),
    Address(UnaryExpression<String>),

    // Maths operations.
    Multiply(BinaryExpression),
    Divide(BinaryExpression),
    Modulo(BinaryExpression),
    Add(BinaryExpression),
    Subtract(BinaryExpression),

    // Boolean relations.
    Equal(BinaryExpression),
    NotEqual(BinaryExpression),
    Greater(BinaryExpression),
    Less(BinaryExpression),
    GreaterEq(BinaryExpression),
    LessEq(BinaryExpression),
    In(BinaryExpression),
    Not(UnaryExpression<Box<Expression>>),

    // Boolean operations.
    Or(BinaryExpression),
    And(BinaryExpression)
}

/// Represents binary-style expression.
///
/// # Example
/// `10 + 2`
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpression {
    /// Location of the parent expression.
    pub loc: Span,
    /// Left expression.
    pub left: Box<Expression>,
    /// Right expression
    pub right: Box<Expression>,
}

impl BinaryExpression {
    pub fn new(start: usize, end: usize, left: Box<Expression>, right: Box<Expression>) -> Self {
        Self {
            loc: Span { start, end },
            left,
            right,
        }
    }
}

/// Represents unary style expression.
#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpression<T> {
    /// Location of the expression
    pub loc: Span,
    /// Element of the expression.
    pub element: T,
}

impl<T> UnaryExpression<T> {
    pub fn new(start: usize, end: usize, element: T) -> Self {
        Self {
            loc: Span { start, end },
            element,
        }
    }
}
