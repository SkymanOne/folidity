use super::Span;
use logos::{Logos, SpannedIter};
use std::{fmt, num::ParseIntError};
use thiserror::Error;

#[derive(Default, Clone, Debug, PartialEq)]
pub enum LogosError {
    #[default]
    InvalidToken,
    InvalidInteger,
}

#[derive(Default, Clone, Debug, PartialEq, Error)]
pub enum LexicalError {
    #[error("Invalid token found, '{1}'")]
    InvalidToken(Span, String),

    #[error("Invalid integer value")]
    InvalidInteger(Span),

    #[error("Invalid else block. Expected block or `if`")]
    InvalidElseBlock(Span),

    #[default]
    #[error("Unknown error occurred")]
    UnknownError,
}

/// Error type returned by calling `lex.slice().parse()` to u8.
impl From<ParseIntError> for LogosError {
    fn from(_: ParseIntError) -> Self {
        LogosError::InvalidInteger
    }
}

/// Spanned location of a specific token.
pub type Spanned<Tok, Loc> = (Loc, Tok, Loc);

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[\t\n\f]+")] // Ignore this regex pattern between tokens
#[logos(error = LogosError)]
pub enum Token<'input> {
    // Type values
    #[regex("[0-9]+", |lex| lex.slice(), priority = 2)]
    Number(&'input str),
    #[regex("([0-9]*[.])?[0-9]+", |lex| lex.slice(), priority = 1)]
    Float(&'input str),
    #[regex("\'[a-zA-Z]\'", |lex| lex.slice().parse().ok())]
    Char(char),
    #[regex("\"[a-zA-Z]+\"", |lex| lex.slice())]
    String(&'input str),
    #[regex("hex\"[a-zA-Z]+\"", |lex| lex.slice())]
    Hex(&'input str),
    #[regex("a\"[a-zA-Z]+\"", |lex| lex.slice())]
    Address(&'input str),
    #[regex("[_a-zA-Z][_0-9a-zA-Z]+", |lex| lex.slice())]
    Identifier(&'input str),
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[token("{")]
    LCurly,
    #[token("}")]
    RCurly,

    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,

    #[token("=")]
    Assign,

    // Math ops
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Modulo,

    #[token("!")]
    Not,

    // Bool relations
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("<=")]
    Leq,
    #[token(">=")]
    Meq,
    #[token("in")]
    In,

    //Bool operations
    #[token("||")]
    Or,
    #[token("&&")]
    And,

    //Types
    #[token("int")]
    IntType,
    #[token("unit")]
    UIntType,
    #[token("float")]
    FloatType,
    #[token("char")]
    CharType,
    #[token("string")]
    StringType,
    #[token("hex")]
    HexType,
    #[token("address")]
    AddressType,
    #[token("bool")]
    BoolType,
    #[token("()")]
    UnitType,

    //Keywords
    #[token("mapping")]
    Mapping,
    #[token("set")]
    Set,
    #[token("list")]
    List,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("state")]
    State,
    #[token("fn")]
    Func,
    #[token("from")]
    From,
    #[token("return")]
    Return,
    #[token("range")]
    Range,
    #[token("for")]
    For,
    #[token("to")]
    To,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("st")]
    St,
    #[token("when")]
    When,
    #[token("pub")]
    Pub,
    #[token("view")]
    View,
    #[token("@init")]
    Init,
    #[token("version")]
    Version,
    #[token("author")]
    Author,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,

    // Misc chars
    #[token("->")]
    Arr,
    #[token(";")]
    SemiCol,
    #[token(":")]
    Col,
    #[token("@")]
    At,
    #[token(":>")]
    Pipe,
    #[token("|")]
    MatchOr,
    #[token(".")]
    Dot,
    #[token("..")]
    DoubleDot,
    #[token(",")]
    Coma,

    //comment
    #[regex(r"#[^\n]*", |lex| lex.slice())]
    Comment(&'input str),
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut word = |s: &str| -> fmt::Result { write!(f, "{s}") };
        match self {
            Token::Number(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::Char(c) => write!(f, "\'{c}'"),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Hex(s) => write!(f, "hex\"{s}\""),
            Token::Address(s) => write!(f, "hex\"{s}\""),
            Token::Identifier(i) => write!(f, "{i}"),
            Token::True => word("true"),
            Token::False => word("false"),
            Token::LParen => word("("),
            Token::RParen => word(")"),
            Token::LCurly => word("{"),
            Token::RCurly => word("}"),
            Token::LAngle => word("<"),
            Token::RAngle => word(">"),
            Token::Assign => word("="),
            Token::Plus => word("+"),
            Token::Minus => word("-"),
            Token::Mul => word("*"),
            Token::Div => word("/"),
            Token::Modulo => word("%"),
            Token::Not => word("!"),
            Token::Eq => word("=="),
            Token::Neq => word("/="),
            Token::Leq => word("<="),
            Token::Meq => word(">="),
            Token::In => word("in"),
            Token::Or => word("||"),
            Token::And => word("&&"),
            Token::IntType => word("int"),
            Token::UIntType => word("unit"),
            Token::FloatType => word("float"),
            Token::CharType => word("char"),
            Token::StringType => word("string"),
            Token::HexType => word("hex"),
            Token::AddressType => word("address"),
            Token::BoolType => word("bool"),
            Token::UnitType => word("()"),
            Token::Mapping => word("mapping"),
            Token::Set => word("set"),
            Token::List => word("list"),
            Token::Struct => word("struct"),
            Token::Enum => word("enum"),
            Token::State => word("state"),
            Token::Func => word("fn"),
            Token::From => word("from"),
            Token::Return => word("return"),
            Token::Range => word("range"),
            Token::For => word("for"),
            Token::To => word("to"),
            Token::If => word("if"),
            Token::Else => word("else"),
            Token::St => word("st"),
            Token::When => word("when"),
            Token::Pub => word("pub"),
            Token::View => word("view"),
            Token::Init => word("@init"),
            Token::Version => word("version"),
            Token::Author => word("author"),
            Token::Let => word("let"),
            Token::Mut => word("mut"),
            Token::Arr => word("->"),
            Token::Col => word(":"),
            Token::SemiCol => word(";"),
            Token::At => word("@"),
            Token::Pipe => word(":>"),
            Token::MatchOr => word("|"),
            Token::Dot => word("."),
            Token::DoubleDot => word(".."),
            Token::Coma => word(","),
            Token::Comment(c) => write!(f, "{c}"),
        }
    }
}

pub struct Lexer<'input> {
    /// Input stream of lexed tokens.
    token_stream: SpannedIter<'input, Token<'input>>,
    /// List of recovered errors.
    errors: &'input mut Vec<LexicalError>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str, errors: &'input mut Vec<LexicalError>) -> Self {
        // the Token::lexer() method is provided by the Logos trait
        Self {
            token_stream: Token::lexer(input).spanned(),
            errors,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<'input>, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((tok_res, span)) = self.token_stream.next() {
            match tok_res {
                Ok(tok) => match tok {
                    Token::Comment(_) => self.next(),
                    _ => Some((span.start, tok, span.end)),
                },
                Err(err) => {
                    self.errors
                        .push(logos_to_lexical_error(&err, &span, &self.token_stream));
                    self.next()
                }
            }
        } else {
            None
        }
    }
}

fn logos_to_lexical_error<'input>(
    error: &LogosError,
    span: &Span,
    tokens: &SpannedIter<'input, Token<'input>>,
) -> LexicalError {
    match error {
        LogosError::InvalidToken => {
            LexicalError::InvalidToken(span.clone(), tokens.slice().to_string())
        }
        LogosError::InvalidInteger => LexicalError::InvalidInteger(span.clone()),
    }
}
