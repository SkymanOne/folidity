use std::fmt::Display;

use derive_more::Display;
use folidity_semantics::ast::TypeVariant;

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub sig: String,
    pub return_size: u64,
}

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
                write!(f, "{}", hex_str)
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
    More,
    #[display(fmt = "b>")]
    BMore,
    #[display(fmt = "<=")]
    LessEq,
    #[display(fmt = "b<=")]
    BLessEq,
    #[display(fmt = ">=")]
    MoreEq,
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
}

pub trait TypeSizeHint {
    /// Hints the compiler the type size if known at compile time.
    fn size_hint(&self) -> Option<u64>;
}

impl TypeSizeHint for TypeVariant {
    fn size_hint(&self) -> Option<u64> {
        match self {
            TypeVariant::Char => Some(8),
            TypeVariant::Address => Some(32),
            TypeVariant::Unit => Some(0),
            TypeVariant::Bool => Some(8),
            TypeVariant::Function(f) => f.returns.size_hint(),
            _ => None,
        }
    }
}
