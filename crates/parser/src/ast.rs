use super::Span;
use derive_node::Node;

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq, Node, Default)]
pub struct Identifier {
    /// Location of the identifier.
    pub loc: Span,
    /// The name of the identifier.
    pub name: String,
}

impl Identifier {
    /// Check if the identifier is particular variable.
    pub fn is(&self, name: &str) -> bool {
        self.name == name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    FunDeclaration(Box<FunctionDeclaration>),
    EnumDeclaration(Box<EnumDeclaration>),
    StructDeclaration(Box<StructDeclaration>),
    ModelDeclaration(Box<ModelDeclaration>),
    StateDeclaration(Box<StateDeclaration>),
    Error(Span),
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Type {
    pub loc: Span,
    pub ty: TypeVariant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeVariant {
    Int,
    Uint,
    Float,
    Char,
    String,
    Hex,
    Address,
    Unit,
    Bool,
    Set(Set),
    List(List),
    Mapping(Mapping),
    Custom(Identifier),
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Set {
    pub ty: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct List {
    pub ty: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, Node, Default)]
pub struct MappingRelation {
    pub loc: Span,
    pub injective: bool,
    pub partial: bool,
    pub surjective: bool,
}

impl MappingRelation {
    pub fn is_bijective(&self) -> bool {
        self.injective && self.surjective
    }
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Mapping {
    pub from_ty: Box<Type>,
    pub relation: MappingRelation,
    pub to_ty: Box<Type>,
}

/// Parameter declaration of the state.
/// `<ident> <ident>?`
#[derive(Clone, Debug, PartialEq, Node)]
pub struct StateParam {
    pub loc: Span,
    /// State type identifier.
    pub ty: Identifier,
    /// Variable name identifier.
    pub name: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Param {
    pub loc: Span,
    /// Type identifier.
    pub ty: Type,
    /// Variable name identifier.
    pub name: Identifier,
    /// Is param mutable.
    pub is_mut: bool,
}

/// View state modifier.
#[derive(Clone, Debug, PartialEq, Node)]
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
    ParamType(Param),
}

impl FuncReturnType {
    /// Return [`TypeVariant`] of a function return type.
    pub fn ty(&self) -> &TypeVariant {
        match self {
            FuncReturnType::Type(ty) => &ty.ty,
            FuncReturnType::ParamType(pty) => &pty.ty.ty,
        }
    }

    pub fn loc(&self) -> &Span {
        match self {
            FuncReturnType::Type(ty) => &ty.loc,
            FuncReturnType::ParamType(param) => &param.loc,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StateBound {
    pub loc: Span,
    /// Original state
    pub from: Option<StateParam>,
    /// Final state
    pub to: Vec<StateParam>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct AccessAttribute {
    pub loc: Span,
    /// Members delimited by `|`
    pub members: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct FunctionDeclaration {
    /// Location span of the function.
    pub loc: Span,
    /// Is it an initializer?
    /// Marked with `@init`
    pub is_init: bool,
    /// Access attribute `@(a | b | c)`
    pub access_attributes: Vec<AccessAttribute>,
    /// Visibility of the function.
    pub vis: FunctionVisibility,
    /// Function return type declaration.
    pub return_ty: FuncReturnType,
    /// Function name.
    pub name: Identifier,
    /// List of parameters.
    pub params: Vec<Param>,
    /// Bounds for the state transition.
    pub state_bound: Option<StateBound>,
    /// Function logical bounds
    pub st_block: Option<StBlock>,
    /// The body of the function.
    pub body: Statement,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct EnumDeclaration {
    /// Location span of the enum.
    pub loc: Span,
    /// Name of the enum.
    pub name: Identifier,
    /// Variants of the enum.
    pub variants: Vec<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StructDeclaration {
    /// Location span of the struct.
    pub loc: Span,
    /// Name of the struct.
    pub name: Identifier,
    /// Fields of the struct.
    pub fields: Vec<Param>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct ModelDeclaration {
    /// Location span of the model.
    pub loc: Span,
    /// Model name.
    pub name: Identifier,
    /// Fields of the model.
    pub fields: Vec<Param>,
    /// A parent model from which fields are inherited.
    pub parent: Option<Identifier>,
    /// Model logical bounds.
    pub st_block: Option<StBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateBody {
    /// Fields are specified manually.
    Raw(Vec<Param>),
    /// Fields are derived from model.
    Model(Identifier),
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StateDeclaration {
    /// Location span of the model.
    pub loc: Span,
    /// Model name.
    pub name: Identifier,
    /// Body of the state. Its fields.
    pub body: Option<StateBody>,
    /// From which state we can transition.
    /// e.g `StateA st`
    pub from: Option<(Identifier, Option<Identifier>)>,
    /// Model logical bounds.
    pub st_block: Option<StBlock>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StBlock {
    pub loc: Span,
    /// List of logic expressions
    pub expr: Expression,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Return {
    pub loc: Span,
    /// List of logic expressions
    pub expr: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Variable(Variable),
    Assign(Assign),
    IfElse(IfElse),
    ForLoop(ForLoop),
    Iterator(Iterator),
    Return(Return),
    Expression(Expression),
    StateTransition(Expression),
    Skip(Span),

    Block(StatementBlock),
    Error(Span),
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StatementBlock {
    pub loc: Span,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Variable {
    pub loc: Span,
    pub names: Vec<Identifier>,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub value: Option<Expression>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Assign {
    pub loc: Span,
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct IfElse {
    pub loc: Span,
    pub condition: Expression,
    pub body: Box<StatementBlock>,
    pub else_part: Option<Box<Statement>>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct ForLoop {
    pub loc: Span,
    pub var: Variable,
    pub condition: Expression,
    pub incrementer: Expression,
    pub body: Box<StatementBlock>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Iterator {
    pub loc: Span,
    pub names: Vec<Identifier>,
    pub list: Expression,
    pub body: Box<StatementBlock>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StructInit {
    pub loc: Span,
    pub name: Identifier,
    pub args: Vec<Expression>,
    /// Autofill fields from partial object
    /// using `..ident` notation.
    pub auto_object: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Variable(Identifier),

    // Literals
    Number(UnaryExpression<String>),
    Boolean(UnaryExpression<bool>),
    Float(UnaryExpression<String>),
    String(UnaryExpression<String>),
    Char(UnaryExpression<char>),
    Hex(UnaryExpression<String>),
    Address(UnaryExpression<String>),
    List(UnaryExpression<Vec<Expression>>),

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
    And(BinaryExpression),

    FunctionCall(FunctionCall),
    MemberAccess(MemberAccess),
    Pipe(BinaryExpression),
    StructInit(StructInit),
}

impl Expression {
    pub fn new_address(start: usize, end: usize, value: &str) -> Self {
        let reg = regex::Regex::new(r#"(a\")([a-zA-Z0-9]+)(\")"#).unwrap();
        let Some((_, [_, address, _])) = reg.captures(value).map(|caps| caps.extract()) else {
            panic!()
        };
        Expression::Address(UnaryExpression::new(start, end, address.to_string()))
    }

    pub fn new_hex(start: usize, end: usize, value: &str) -> Self {
        let reg = regex::Regex::new(r#"(hex\")([a-zA-Z0-9]+)(\")"#).unwrap();
        let Some((_, [_, hex, _])) = reg.captures(value).map(|caps| caps.extract()) else {
            panic!()
        };
        Expression::Hex(UnaryExpression::new(start, end, hex.to_string()))
    }

    pub fn new_string(start: usize, end: usize, value: &str) -> Self {
        let reg = regex::Regex::new(r#"(s\")([\w\W][^"]*)(\")"#).unwrap();
        let Some((_, [_, string, _])) = reg.captures(value).map(|caps| caps.extract()) else {
            panic!()
        };
        Expression::String(UnaryExpression::new(start, end, string.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct FunctionCall {
    /// Location of the parent expression.
    pub loc: Span,
    /// Name of the function.
    pub name: Identifier,
    /// List of arguments.
    pub args: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct MemberAccess {
    /// Location of the parent expression.
    pub loc: Span,
    /// Expression to access the member from
    pub expr: Box<Expression>,
    /// List of arguments.
    pub member: Identifier,
}

/// Represents binary-style expression.
///
/// # Example
/// `10 + 2`
#[derive(Clone, Debug, PartialEq, Node)]
pub struct BinaryExpression {
    /// Location of the parent expression.
    pub loc: Span,
    /// Left expression.
    pub left: Box<Expression>,
    /// Right expression
    pub right: Box<Expression>,
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

impl Expression {
    pub fn loc(&self) -> &Span {
        match self {
            Expression::Variable(i) => &i.loc,
            Expression::Number(u) => &u.loc,
            Expression::Boolean(u) => &u.loc,
            Expression::Float(u) => &u.loc,
            Expression::String(u) => &u.loc,
            Expression::Char(u) => &u.loc,
            Expression::Hex(u) => &u.loc,
            Expression::Address(u) => &u.loc,
            Expression::List(u) => &u.loc,
            Expression::Multiply(b) => &b.loc,
            Expression::Divide(b) => &b.loc,
            Expression::Modulo(b) => &b.loc,
            Expression::Add(b) => &b.loc,
            Expression::Subtract(b) => &b.loc,
            Expression::Equal(b) => &b.loc,
            Expression::NotEqual(b) => &b.loc,
            Expression::Greater(b) => &b.loc,
            Expression::Less(b) => &b.loc,
            Expression::GreaterEq(b) => &b.loc,
            Expression::LessEq(b) => &b.loc,
            Expression::In(b) => &b.loc,
            Expression::Not(u) => &u.loc,
            Expression::Or(b) => &b.loc,
            Expression::And(b) => &b.loc,
            Expression::FunctionCall(f) => &f.loc,
            Expression::MemberAccess(m) => &m.loc,
            Expression::Pipe(b) => &b.loc,
            Expression::StructInit(s) => &s.loc,
        }
    }
}

impl Statement {
    pub fn loc(&self) -> &Span {
        match self {
            Statement::Variable(v) => &v.loc,
            Statement::Assign(a) => &a.loc,
            Statement::IfElse(br) => &br.loc,
            Statement::ForLoop(l) => &l.loc,
            Statement::Iterator(i) => &i.loc,
            Statement::Return(e) => &e.loc,
            Statement::Expression(e) => e.loc(),
            Statement::StateTransition(tr) => tr.loc(),
            Statement::Block(b) => &b.loc,
            Statement::Skip(s) => s,
            Statement::Error(s) => s,
        }
    }
}
