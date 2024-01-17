use logos::{Logos, SpannedIter};
use std::{fmt, num::ParseIntError, ops::Range};
use thiserror::Error;

type Span = Range<usize>;

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
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
#[logos(error = LogosError)]
pub enum Token {
    // Type values
    #[regex("[0-9]+", |lex| lex.slice().parse().ok(), priority = 2)]
    Number(String),
    #[regex("([0-9]*[.])?[0-9]+", |lex| lex.slice().parse().ok(), priority = 1)]
    Float(f64),
    #[regex("\'[a-zA-Z]\'", |lex| lex.slice().parse().ok())]
    Char(char),
    #[regex("\"[a-zA-Z]+\"", |lex| lex.slice().parse().ok())]
    String(String),
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
    #[token("hash")]
    HashType,
    #[token("address")]
    AddressType,
    #[token("bool")]
    BoolType,
    #[token("()")]
    UnitType,

    //Keywords
    #[token("Mapping")]
    Mapping,
    #[token("Set")]
    Set,
    #[token("List")]
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
    #[token("\"")]
    DoubleQuote,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut word = |s: &str| -> fmt::Result {
            write!(f, "{s}")
        };
        match self {
            Token::Number(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::Char(c) => write!(f, "\'{c}'"),
            Token::String(s) => write!(f, "\"{s}\""),
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
            Token::Eq => word("=="),
            Token::Neq => word("/="),
            Token::Leq => word("<="),
            Token::Meq => word(">="),
            Token::In => word("in"),
            Token::IntType => word("int"),
            Token::UIntType => word("unit"),
            Token::FloatType => word("float"),
            Token::CharType => word("char"),
            Token::StringType => word("string"),
            Token::HashType => word("hash"),
            Token::AddressType => word("address"),
            Token::BoolType => word("bool"),
            Token::UnitType => word("()"),
            Token::Mapping => word("Mapping"),
            Token::Set => word("Set"),
            Token::List => word("List"),
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
            Token::DoubleQuote => word("\"")
        }
    }
}

pub struct Lexer<'input> {
    /// Input stream of lexed tokens.
    token_stream: SpannedIter<'input, Token>,
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
    type Item = Spanned<Token, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((tok_res, span)) = self.token_stream.next() {
            match tok_res {
                Ok(tok) => Some((span.start, tok, span.end)),
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

fn logos_to_lexical_error(
    error: &LogosError,
    span: &Span,
    tokens: &SpannedIter<'_, Token>,
) -> LexicalError {
    match error {
        LogosError::InvalidToken => {
            LexicalError::InvalidToken(span.clone(), tokens.slice().to_string())
        }
        LogosError::InvalidInteger => LexicalError::InvalidInteger(span.clone()),
    }
}
