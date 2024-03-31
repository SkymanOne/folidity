use std::{
    fmt::Display,
    ops::Range,
};

pub type Span = Range<usize>;

pub use yansi::{
    Color,
    Paint,
};

pub fn disable_pretty_print() {
    yansi::disable();
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Lexer,
    Parser,
    Semantics,
    Type,
    Verification,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut word = |s: &str| -> std::fmt::Result { write!(f, "{s}") };
        match self {
            ErrorType::Lexer => word("Lexical error"),
            ErrorType::Parser => word("Parser error"),
            ErrorType::Semantics => word("Semantic error"),
            ErrorType::Type => word("Type error"),
            ErrorType::Verification => word("Verification error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Level {
    Info,
    Warning,
    Error,
}

impl<'a> From<Level> for ariadne::ReportKind<'a> {
    fn from(val: Level) -> Self {
        match &val {
            Level::Info => Self::Advice,
            Level::Warning => Self::Warning,
            Level::Error => Self::Error,
        }
    }
}

/// Error report.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Location of an error
    pub loc: Span,
    /// A type of an error to occur.
    pub error_type: ErrorType,
    /// Level of an error.
    pub level: Level,
    /// Message of an error
    pub message: String,

    pub additional_info: Vec<Report>,
}

impl Report {
    /// Build a report from the lexer error.
    pub fn lexer_error(loc: Span, message: String) -> Self {
        Self {
            loc,
            error_type: ErrorType::Lexer,
            level: Level::Error,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the parser error.
    pub fn parser_error(start: usize, end: usize, message: String) -> Self {
        Self {
            loc: Span { start, end },
            error_type: ErrorType::Parser,
            level: Level::Error,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the semantic error.
    pub fn semantic_error(loc: Span, message: String) -> Self {
        Self {
            loc,
            error_type: ErrorType::Semantics,
            level: Level::Error,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the semantic warning.
    pub fn semantic_warning(loc: Span, message: String) -> Self {
        Self {
            loc,
            error_type: ErrorType::Semantics,
            level: Level::Warning,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the type error.
    pub fn type_error(loc: Span, message: String) -> Self {
        Self {
            loc,
            error_type: ErrorType::Type,
            level: Level::Error,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the verification error.
    pub fn ver_error(loc: Span, message: String) -> Self {
        Self {
            loc,
            error_type: ErrorType::Verification,
            level: Level::Error,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the verification suggestion.
    pub fn ver_info(message: String) -> Self {
        Self {
            loc: Default::default(),
            error_type: ErrorType::Verification,
            level: Level::Info,
            message,
            additional_info: vec![],
        }
    }

    /// Build a report from the verification error with additional info.
    pub fn ver_error_with_extra(loc: Span, message: String, errs: Vec<Report>) -> Self {
        Self {
            loc,
            error_type: ErrorType::Verification,
            level: Level::Error,
            message,
            additional_info: errs,
        }
    }
}
