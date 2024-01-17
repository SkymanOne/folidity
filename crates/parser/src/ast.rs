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
pub enum Expression {
    Number(UnaryExpression<String>),

    Multiply(BinaryExpression),
    Divide(BinaryExpression),
    Add(BinaryExpression),
    Subtract(BinaryExpression),
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
