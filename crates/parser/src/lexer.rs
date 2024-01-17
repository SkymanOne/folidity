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
    // Tokens can be literal strings, of any length.
    #[token("version:")]
    Version,

    #[token(".")]
    Period,

    #[token("\"")]
    Quote,

    #[token("author:")]
    Author,

    // Type values
    #[regex("[0-9]+", |lex| lex.slice().parse().ok(), priority = 2)]
    Integer(i128),

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

    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Version => write!(f, "version:"),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Period => write!(f, "."),
            Token::Quote => write!(f, "\""),
            Token::String(s) => write!(f, "{}", s),
            Token::Author => write!(f, "author:"),
            Token::Float(n) => write!(f, "float{}", n),
            Token::Char(c) => write!(f, "{}", c),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LCurly => write!(f, "{{"),
            Token::RCurly => write!(f, "}}"),
            Token::Assign => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            
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
