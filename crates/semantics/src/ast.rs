use std::collections::HashSet;

use derive_node::Node;
use folidity_diagnostics::Report;
use folidity_parser::{
    ast::{BinaryExpression, Identifier, MappingRelation, UnaryExpression},
    Span,
};
use indexmap::IndexMap;

use crate::{
    contract::ContractDefinition,
    global_symbol::{GlobalSymbol, SymbolInfo},
};

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Type {
    pub loc: Span,
    pub ty: TypeVariant,
}

impl Type {
    /// Is data type primitive.
    pub fn is_primitive(&self) -> bool {
        matches!(
            &self.ty,
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
    pub fn custom_type_dependencies(&self, contract: &mut ContractDefinition) -> HashSet<usize> {
        match &self.ty {
            TypeVariant::Set(s) => s.ty.custom_type_dependencies(contract),
            TypeVariant::List(s) => s.ty.custom_type_dependencies(contract),
            TypeVariant::Mapping(m) => {
                let mut set = m.from_ty.custom_type_dependencies(contract);
                set.extend(m.to_ty.custom_type_dependencies(contract));
                set
            }
            TypeVariant::Struct(s) => HashSet::from([s.i]),
            _ => HashSet::new(),
        }
    }
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
    Function(SymbolInfo),
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionType {
    params: Vec<TypeVariant>,
    returns: TypeVariant,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct Set {
    pub ty: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, Node)]
pub struct List {
    pub ty: Box<Type>,
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
    pub ty: Option<Identifier>,
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
    Variable(Identifier),

    Number(UnaryExpression<String>),
    Boolean(UnaryExpression<bool>),
    Float(UnaryExpression<String>),
    String(UnaryExpression<String>),
    Char(UnaryExpression<char>),
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
    And(BinaryExpression),

    FunctionCall(FunctionCall),
    MemberAccess(MemberAccess),
    Pipe(BinaryExpression),
    StructInit(UnaryExpression<StructInit>),

    List(UnaryExpression<Vec<Expression>>),
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
}
