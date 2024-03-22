use std::{
    collections::HashSet,
    fmt::Display,
};

use derive_node::Node;
use folidity_parser::{
    ast::{
        Identifier,
        MappingRelation,
    },
    Span,
};
use indexmap::IndexMap;
use num_bigint::{
    BigInt,
    BigUint,
};
use num_rational::BigRational;

use crate::{
    global_symbol::SymbolInfo,
    symtable::Scope,
};
use algonaut_core::Address;

#[derive(Clone, Debug, PartialEq, Node, Default)]
pub struct Type {
    pub loc: Span,
    pub ty: TypeVariant,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TypeVariant {
    #[default]
    Int,
    Uint,
    Float,
    Char,
    String,
    Hex,
    Address,
    Unit,
    Bool,
    Set(Box<TypeVariant>),
    List(Box<TypeVariant>),
    Mapping(Mapping),
    Function(FunctionType),
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),

    // A placeholder for generics.
    // Mainly used in for built-in list function
    // e.g. `map`, `filter`, etc.
    // which can operate on lists of generic types.
    // It hold a list of possible concrete types allowed for this generic.
    Generic(Vec<TypeVariant>),
}

impl TypeVariant {
    /// Is data type primitive.
    pub fn is_primitive(&self) -> bool {
        matches!(
            &self,
            TypeVariant::Int
                | TypeVariant::Uint
                | TypeVariant::Float
                | TypeVariant::Char
                | TypeVariant::String
                | TypeVariant::Hex
                | TypeVariant::Address
                | TypeVariant::Unit
                | TypeVariant::Bool
        )
    }

    /// Find the set of dependent user defined types that are encapsulated by this type.
    pub fn custom_type_dependencies(&self) -> HashSet<usize> {
        match &self {
            TypeVariant::Set(ty) => ty.custom_type_dependencies(),
            TypeVariant::List(ty) => ty.custom_type_dependencies(),
            TypeVariant::Mapping(m) => {
                let mut set = m.from_ty.custom_type_dependencies();
                set.extend(m.to_ty.custom_type_dependencies());
                set
            }
            TypeVariant::Struct(s) => HashSet::from([s.i]),
            _ => HashSet::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct FunctionType {
    pub params: Vec<TypeVariant>,
    pub returns: Box<TypeVariant>,
}

#[derive(Clone, Debug, PartialEq, Node, Default)]
pub struct Set {
    pub ty: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct List {
    pub ty: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, Node, Default)]
pub struct Mapping {
    pub from_ty: Box<TypeVariant>,
    pub relation: MappingRelation,
    pub to_ty: Box<TypeVariant>,
}

/// Parameter declaration of the state.
/// `<ident> <ident>?`
#[derive(Clone, Debug, PartialEq, Node)]
pub struct StateParam {
    pub loc: Span,
    /// State type identifier.
    pub ty: SymbolInfo,
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
    /// Is the field recursive.
    pub recursive: bool,
}

/// View state modifier.
#[derive(Clone, Debug, PartialEq, Node)]
pub struct ViewState {
    pub loc: Span,
    /// State type identifier.
    pub ty: usize,
    /// Variable name identifier.
    pub name: Identifier,
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
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StateBound {
    pub loc: Span,
    /// Original state
    pub from: Option<StateParam>,
    /// Final state
    pub to: Vec<StateParam>,
}

#[derive(Clone, Debug)]
pub struct Function {
    /// Location span of the function.
    pub loc: Span,
    /// Is it an initializer?
    /// Marked with `@init`
    pub is_init: bool,
    /// Access attribute `@(a | b | c)`
    pub access_attributes: Vec<Expression>,
    /// Visibility of the function.
    pub vis: FunctionVisibility,
    /// Function return type declaration.
    pub return_ty: FuncReturnType,
    /// Function name.
    pub name: Identifier,
    /// List of parameters.
    pub params: IndexMap<String, Param>,
    /// Function logical bounds.
    pub bounds: Vec<Expression>,
    /// Bounds for the state transition.
    pub state_bound: Option<StateBound>,
    /// The body of the function.
    pub body: Vec<Statement>,
    /// Scope table for the function context.
    pub scope: Scope,
}

impl Function {
    pub fn new(
        loc: Span,
        is_init: bool,
        vis: FunctionVisibility,
        return_ty: FuncReturnType,
        name: Identifier,
        params: IndexMap<String, Param>,
        state_bound: Option<StateBound>,
    ) -> Self {
        Function {
            loc,
            is_init,
            access_attributes: Vec::new(),
            vis,
            return_ty,
            name,
            params,
            state_bound,
            body: Vec::new(),
            bounds: Vec::new(),
            scope: Scope::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDeclaration {
    /// Location span of the enum.
    pub loc: Span,
    /// Name of the enum.
    pub name: Identifier,
    /// Variants of the enum.
    pub variants: IndexMap<String, Span>,
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
    /// Identified as a index in the global symbol table.
    pub parent: Option<usize>,
    /// Model logical bounds.
    pub bounds: Vec<Expression>,
    /// Is the parent model recursive.
    pub recursive_parent: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateBody {
    /// Fields are specified manually.
    Raw(Vec<Param>),
    /// Fields are derived from model.
    Model(SymbolInfo),
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
    pub from: Option<(SymbolInfo, Option<Identifier>)>,
    /// Model logical bounds.
    pub bounds: Vec<Expression>,
    /// Is the parent state recursive.
    pub recursive_parent: bool,
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

    Block(StatementBlock),
    Skip(Span),
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
    pub pos: usize,
    pub mutable: bool,
    pub ty: TypeVariant,
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
    pub body: Vec<Statement>,
    pub else_part: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct ForLoop {
    pub loc: Span,
    pub var: Box<Statement>,
    pub condition: Expression,
    pub incrementer: Expression,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Iterator {
    pub loc: Span,
    pub names: Vec<Identifier>,
    pub list: Expression,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StructInit {
    pub loc: Span,
    pub name: Identifier,
    pub args: Vec<Expression>,
    /// Autofill fields from partial object
    /// using `..ident` notation.
    pub auto_object: Option<Identifier>,
    /// Type of an expression.
    pub ty: TypeVariant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Variable(UnaryExpression<usize>),

    // Literals
    Int(UnaryExpression<BigInt>),
    UInt(UnaryExpression<BigUint>),
    Float(UnaryExpression<BigRational>),
    Boolean(UnaryExpression<bool>),
    String(UnaryExpression<String>),
    Char(UnaryExpression<char>),
    Hex(UnaryExpression<Vec<u8>>),
    Address(UnaryExpression<Address>),
    Enum(UnaryExpression<usize>),

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
    StructInit(StructInit),

    List(UnaryExpression<Vec<Expression>>),
}

/// Represents unary style expression.
#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpression<T> {
    /// Location of the expression
    pub loc: Span,
    /// Element of the expression.
    pub element: T,
    /// Type of an expression.
    pub ty: TypeVariant,
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
    /// Type of an expression.
    pub ty: TypeVariant,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct FunctionCall {
    /// Location of the parent expression.
    pub loc: Span,
    /// Name of the function.
    pub name: Identifier,
    /// List of arguments.
    pub args: Vec<Expression>,
    pub returns: TypeVariant,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct MemberAccess {
    /// Location of the parent expression.
    pub loc: Span,
    /// Expression to access the member from
    pub expr: Box<Expression>,
    /// Member of a struct.
    pub member: (usize, Span),
    /// Type of an expression.
    pub ty: TypeVariant,
}

impl Display for TypeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut word = |s: &str| -> std::fmt::Result { write!(f, "{s}") };
        match self {
            TypeVariant::Int => word("int"),
            TypeVariant::Uint => word("uint"),
            TypeVariant::Float => word("float"),
            TypeVariant::Char => word("char"),
            TypeVariant::String => word("string"),
            TypeVariant::Hex => word("hex"),
            TypeVariant::Address => word("address"),
            TypeVariant::Unit => word("unit"),
            TypeVariant::Bool => word("bool"),
            TypeVariant::Set(_) => word("set"),
            TypeVariant::List(_) => word("list"),
            TypeVariant::Mapping(_) => word("mapping"),
            TypeVariant::Function(_) => word("function"),
            TypeVariant::Struct(_) => word("struct"),
            TypeVariant::Model(_) => word("model"),
            TypeVariant::Enum(_) => word("enum"),
            TypeVariant::State(_) => word("state"),
            TypeVariant::Generic(_) => word("generic type"),
        }
    }
}

/// Extracts literal value, `T`, from the expression, if possible.
pub trait TryGetValue<T> {
    fn try_get(&self) -> Result<T, ()>;
}

impl Expression {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::Int(_)
                | Expression::UInt(_)
                | Expression::Float(_)
                | Expression::Char(_)
                | Expression::String(_)
                | Expression::Hex(_)
                | Expression::Address(_)
                | Expression::Boolean(_)
        )
    }

    /// Check if the expression is a wildcard `any` variable.
    pub fn is_access_wildcard(&self, scope: &Scope) -> bool {
        if let Expression::Variable(var) = self {
            scope
                .find_symbol(&var.element)
                .map_or(false, |s| s.ident.is("any"))
        } else {
            false
        }
    }
}

impl TryGetValue<BigInt> for Expression {
    fn try_get(&self) -> Result<BigInt, ()> {
        match self {
            Expression::Int(e) => Ok(e.element.clone()),
            _ => Err(()),
        }
    }
}

impl TryGetValue<BigUint> for Expression {
    fn try_get(&self) -> Result<BigUint, ()> {
        match self {
            Expression::UInt(e) => Ok(e.element.clone()),
            _ => Err(()),
        }
    }
}

impl TryGetValue<BigRational> for Expression {
    fn try_get(&self) -> Result<BigRational, ()> {
        match self {
            Expression::Float(e) => Ok(e.element.clone()),
            _ => Err(()),
        }
    }
}

impl TryGetValue<String> for Expression {
    fn try_get(&self) -> Result<String, ()> {
        match self {
            Expression::String(e) => Ok(e.element.clone()),
            _ => Err(()),
        }
    }
}

impl TryGetValue<char> for Expression {
    fn try_get(&self) -> Result<char, ()> {
        match self {
            Expression::Char(e) => Ok(e.element),
            _ => Err(()),
        }
    }
}

impl TryGetValue<Address> for Expression {
    fn try_get(&self) -> Result<Address, ()> {
        match self {
            Expression::Address(e) => Ok(e.element),
            _ => Err(()),
        }
    }
}

impl TryGetValue<bool> for Expression {
    fn try_get(&self) -> Result<bool, ()> {
        match self {
            Expression::Boolean(e) => Ok(e.element),
            _ => Err(()),
        }
    }
}

impl TryGetValue<Vec<u8>> for Expression {
    fn try_get(&self) -> Result<Vec<u8>, ()> {
        match self {
            Expression::Hex(e) => Ok(e.element.clone()),
            _ => Err(()),
        }
    }
}
