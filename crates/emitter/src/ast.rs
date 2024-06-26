use std::fmt::Display;

use derive_more::Display;
use folidity_semantics::{
    ast::{
        Param,
        TypeVariant,
    },
    ContractDefinition,
};

/// Represents a constant literal in teal bytecode.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Uint(u64),
    Bytes(Vec<u8>),
    String(String),
    StringLit(String),
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Uint(n) => write!(f, "{}", n),
            Constant::Bytes(b) => {
                let hex_str = hex::encode(b);
                write!(f, "0x{}", hex_str)
            }
            Constant::String(s) => write!(f, "\"{}\"", s),
            Constant::StringLit(s) => write!(f, "{}", s),
        }
    }
}

/// Represents a chunk of code of teal AVM bytecode.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub op: Instruction,
    pub constants: Vec<Constant>,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let constants_str = self
            .constants
            .iter()
            .fold(String::new(), |init, x| format!("{init} {x}"))
            .trim()
            .to_string();
        write!(f, "{} {}", self.op, constants_str)
    }
}

impl Chunk {
    pub fn new_empty(op: Instruction) -> Self {
        Self {
            op,
            constants: vec![],
        }
    }
    pub fn new_single(op: Instruction, c: Constant) -> Self {
        Self {
            op,
            constants: vec![c],
        }
    }

    pub fn new_multiple(op: Instruction, cs: Vec<Constant>) -> Self {
        Self { op, constants: cs }
    }
}

/// Represents AVM teal opcode from https://developer.algorand.org/docs/get-details/dapps/avm/teal/opcodes/v10/
#[derive(Debug, Clone, Display, PartialEq)]
pub enum Instruction {
    #[display(fmt = "")]
    Empty,

    #[display(fmt = "+")]
    Plus,
    #[display(fmt = "b+")]
    BPlus,
    #[display(fmt = "-")]
    Minus,
    #[display(fmt = "b-")]
    BMinus,
    #[display(fmt = "*")]
    Mul,
    #[display(fmt = "b*")]
    BMul,
    #[display(fmt = "/")]
    Div,
    #[display(fmt = "b/")]
    BDiv,
    #[display(fmt = "<")]
    Less,
    #[display(fmt = "b<")]
    BLess,
    #[display(fmt = ">")]
    Greater,
    #[display(fmt = "b>")]
    BMore,
    #[display(fmt = "<=")]
    LessEq,
    #[display(fmt = "b<=")]
    BLessEq,
    #[display(fmt = ">=")]
    GreaterEq,
    #[display(fmt = "b>=")]
    BMoreEq,
    #[display(fmt = "&&")]
    And,
    #[display(fmt = "||")]
    Or,
    #[display(fmt = "==")]
    Eq,
    #[display(fmt = "b==")]
    BEq,
    #[display(fmt = "!=")]
    Neq,
    #[display(fmt = "b!=")]
    BNeq,
    #[display(fmt = "!")]
    Not,
    #[display(fmt = "len")]
    Len,
    #[display(fmt = "&")]
    Mod,
    #[display(fmt = "b&")]
    BMod,
    #[display(fmt = "concat")]
    Concat,

    #[display(fmt = "pushint")]
    PushInt,
    #[display(fmt = "pushbytes")]
    PushBytes,
    #[display(fmt = "addr")]
    PushAddr,

    #[display(fmt = "bzero")]
    ArrayInit,
    #[display(fmt = "store")]
    Store,
    #[display(fmt = "load")]
    Load,
    #[display(fmt = "replace")]
    Replace,
    #[display(fmt = "extract")]
    Extract,
    #[display(fmt = "extract3")]
    Extract3,
    #[display(fmt = "extract_uint64")]
    ExtractUint,

    #[display(fmt = "callsub")]
    CallSub,

    #[display(fmt = "assert")]
    Assert,
    #[display(fmt = "err")]
    Error,
    #[display(fmt = "itob")]
    Itob,
    #[display(fmt = "dup")]
    Dup,
    #[display(fmt = "{}:", _0)]
    Label(String),
    #[display(fmt = "retsub")]
    ReturnSubroutine,

    #[display(fmt = "txn")]
    Txn,
    #[display(fmt = "txna")]
    Txna,
    #[display(fmt = "global")]
    Global,

    #[display(fmt = "box_get")]
    BoxGet,
    #[display(fmt = "box_put")]
    BoxPut,

    #[display(fmt = "b")]
    Branch,
    #[display(fmt = "bnz")]
    BranchNotZero,
    #[display(fmt = "bz")]
    BranchZero,

    #[display(fmt = "return")]
    Return,
    #[display(fmt = "log")]
    Log,
    #[display(fmt = "len")]
    Length,
}

pub trait TypeSizeHint {
    /// Hints the compiler the type size if known at compile time.
    fn size_hint(&self, contract: &ContractDefinition) -> u64;
}

impl TypeSizeHint for TypeVariant {
    fn size_hint(&self, contract: &ContractDefinition) -> u64 {
        match self {
            TypeVariant::Char | TypeVariant::Bool | TypeVariant::Uint | TypeVariant::Float => 8,
            TypeVariant::Int => 16,
            TypeVariant::Address => 32,
            TypeVariant::Unit => 0,
            TypeVariant::Enum(_) => 16,
            TypeVariant::Function(f) => f.returns.size_hint(contract),
            TypeVariant::Set(_)
            | TypeVariant::List(_)
            | TypeVariant::Mapping(_)
            | TypeVariant::String
            | TypeVariant::Hex => 512,
            TypeVariant::Struct(sym) => {
                let struct_decl = &contract.structs[sym.i];
                struct_size(&struct_decl.fields, contract)
            }
            TypeVariant::Model(sym) => {
                let model_decl = &contract.models[sym.i];
                struct_size(&model_decl.fields(contract), contract)
            }
            TypeVariant::State(sym) => {
                let state_decl = &contract.states[sym.i];
                struct_size(&state_decl.fields(contract), contract)
            }
            TypeVariant::Generic(_) => unimplemented!(),
        }
    }
}

pub fn struct_size(fields: &[Param], contract: &ContractDefinition) -> u64 {
    // construct array
    let mut array_size: u64 = 0;
    for f in fields {
        array_size += f.ty.ty.size_hint(contract);

        if f.ty.ty.is_resizable() {
            array_size += 8; // reserve one more uint64 block for actual size of
                             // resizeable struct.
        }
    }

    array_size
}
