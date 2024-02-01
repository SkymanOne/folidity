use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone)]
pub enum ErrorType {
    Lexer,
    Parser,
    Semantics,
    Type,
    Functional,
}

#[derive(Debug, Clone)]
pub enum Level {
    Info,
    Warning,
    Error,
}

/// Error report.
#[derive(Debug, Clone)]
pub struct Report {
    /// Location of an error
    pub loc: Span,
    /// A type of an error to occur.
    pub error_type: ErrorType,
    /// Level of an error.
    pub level: Level,
    /// Message of an error
    pub message: String,
}

impl Report {
    /// Build report from the lexer error.
    pub fn lexer_error(loc: Span, message: String) -> Self {
        Self {
            loc,
            error_type: ErrorType::Lexer,
            level: Level::Error,
            message,
        }
    }

    /// Build report from the parser error.
    pub fn parser_error(start: usize, end: usize, message: String) -> Self {
        Self {
            loc: Span { start, end },
            error_type: ErrorType::Parser,
            level: Level::Error,
            message,
        }
    }
}