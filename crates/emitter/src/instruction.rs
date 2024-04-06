use std::fmt::Display;

use derive_more::Display;

/// Represents a constant literal in teal bytecode.
#[derive(Debug, Clone)]
pub enum Constant {
    Uint(u64),
    Bytes(Vec<u8>),
    String(String),
    Addr(String),
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
            Constant::Addr(s) => write!(f, "{}", s),
        }
    }
}

/// Represents a chunk of code of teal AVM bytecode.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Display)]
pub enum Instruction {
    #[display(fmt = "+")]
    Plus,
    #[display(fmt = "-")]
    Minus,
    #[display(fmt = "/")]
    Div,
    #[display(fmt = "<")]
    Less,
    #[display(fmt = ">")]
    More,
    #[display(fmt = "<=")]
    LessEq,
    #[display(fmt = ">=")]
    MoreEq,
    #[display(fmt = "&&")]
    And,
    #[display(fmt = "||")]
    Or,
    #[display(fmt = "==")]
    Eq,
    #[display(fmt = "!=")]
    Neq,
    #[display(fmt = "!")]
    Not,
    #[display(fmt = "len")]
    Len,
    #[display(fmt = "&")]
    Mod,
    // todo: bitwise ops
    // here
    #[display(fmt = "pushint")]
    PushInt,
    #[display(fmt = "pushbytes")]
    PushBytes,
    #[display(fmt = "addr")]
    PushAddr,

    #[display(fmt = "store")]
    Store,
    #[display(fmt = "load")]
    Load,
}
