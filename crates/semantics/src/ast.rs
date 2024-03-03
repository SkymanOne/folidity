use std::{collections::HashSet, fmt::Display};

use derive_node::Node;
use folidity_parser::{
    ast::{Identifier, MappingRelation},
    Span,
};
use indexmap::IndexMap;
use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;

use crate::{global_symbol::SymbolInfo, symtable::SymTable};
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
    Function(SymbolInfo),
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),
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

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionType {
    params: Vec<TypeVariant>,
    returns: TypeVariant,
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
    pub ty: usize,
    /// Variable name identifier.
    pub name: Identifier,
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

#[derive(Clone, Debug, PartialEq, Node)]
pub struct StateBound {
    pub loc: Span,
    /// Original state
    pub from: StateParam,
    /// Final state
    pub to: Vec<StateParam>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct AccessAttribute {
    pub loc: Span,
    /// Members delimited by `|`
    pub members: Vec<Expression>,
}

#[derive(Clone, Debug)]
pub struct Function {
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
    pub params: IndexMap<String, Param>,
    /// Function logical bounds.
    pub bounds: Vec<Expression>,
    /// The body of the function.
    pub body: Vec<Statement>,
    /// Symbol table for the function context.
    pub symtable: SymTable,
}

impl Function {
    pub fn new(
        loc: Span,
        is_init: bool,
        vis: FunctionVisibility,
        return_ty: FuncReturnType,
        name: Identifier,
        params: IndexMap<String, Param>,
    ) -> Self {
        Function {
            loc,
            is_init,
            access_attributes: Vec::new(),
            vis,
            return_ty,
            name,
            params,
            body: Vec::new(),
            bounds: Vec::new(),
            symtable: SymTable::default(),
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
    pub from: Option<(usize, Option<Identifier>)>,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Variable(Variable),
    Assign(Assign),
    IfElse(IfElse),
    ForLoop(ForLoop),
    Iterator(Iterator),
    Return(Expression),
    FunCall(FunctionCall),
    StateTransition(StructInit),

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
    StructInit(UnaryExpression<StructInit>),

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
    /// List of arguments.
    pub member: Identifier,
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
        }
    }
}

impl Expression {
    ///  Retrieves type from the expression.
    pub fn ty(&self) -> &TypeVariant {
        match self {
            Expression::Variable(e) => &e.ty,
            Expression::Int(e) => &e.ty,
            Expression::UInt(e) => &e.ty,
            Expression::Float(e) => &e.ty,
            Expression::Boolean(e) => &e.ty,
            Expression::String(e) => &e.ty,
            Expression::Char(e) => &e.ty,
            Expression::Hex(e) => &e.ty,
            Expression::Address(e) => &e.ty,
            Expression::Multiply(e) => &e.ty,
            Expression::Divide(e) => &e.ty,
            Expression::Modulo(e) => &e.ty,
            Expression::Add(e) => &e.ty,
            Expression::Subtract(e) => &e.ty,
            Expression::Equal(e) => &e.ty,
            Expression::NotEqual(e) => &e.ty,
            Expression::Greater(e) => &e.ty,
            Expression::Less(e) => &e.ty,
            Expression::GreaterEq(e) => &e.ty,
            Expression::LessEq(e) => &e.ty,
            Expression::In(e) => &e.ty,
            Expression::Not(e) => &e.ty,
            Expression::Or(e) => &e.ty,
            Expression::And(e) => &e.ty,
            Expression::FunctionCall(e) => &e.returns,
            Expression::MemberAccess(e) => &e.ty,
            Expression::Pipe(e) => &e.ty,
            Expression::StructInit(e) => &e.ty,
            Expression::List(e) => &e.ty,
        }
    }
}
