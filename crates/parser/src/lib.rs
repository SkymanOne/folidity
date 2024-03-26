pub mod ast;
pub mod lexer;

use ast::Source;
use folidity_diagnostics::Report;
use lalrpop_util::{
    lalrpop_mod,
    ErrorRecovery,
    ParseError,
};
use lexer::{
    Lexer,
    LexicalError,
    Token,
};
use std::ops::Range;

#[cfg(test)]
mod tests;

pub type Span = Range<usize>;

lalrpop_mod!(pub folidity);

/// Parses a Folidity file into a concrete syntax tree.
/// # Returns
///
/// - A root of the syntax tree [`Source`]
///
/// # Errors
///
/// - A list of [`Report`] diagnostic error
pub fn parse(src: &str) -> Result<Source, Vec<Report>> {
    let mut lexer_errors = Vec::new();
    let tokens = Lexer::new(src, &mut lexer_errors);
    let mut parser_errors: Vec<ErrorRecovery<usize, Token, LexicalError>> = Vec::new();
    let res = folidity::FolidityTreeParser::new().parse(&mut parser_errors, tokens);

    let mut reports: Vec<Report> = Vec::new();

    for pe in &parser_errors {
        reports.push(parser_error_to_report(&pe.error));
    }

    match res {
        Err(e) => {
            reports.push(parser_error_to_report(&e));
            Err(reports)
        }
        // Ok(_) if !reports.is_empty() => Err(reports),
        Ok(mut tree) => {
            tree.diagnostics.extend(reports);
            Ok(tree)
        }
    }
}

impl From<LexicalError> for Report {
    fn from(value: LexicalError) -> Self {
        match value {
            LexicalError::InvalidToken(l) => {
                Report::lexer_error(l, "Invalid token present".to_string())
            }
            LexicalError::InvalidInteger(l) => {
                Report::lexer_error(l, "Invalid integer present".to_string())
            }
            LexicalError::InvalidElseBlock(l) => {
                Report::lexer_error(l, "Invalid branch block".to_string())
            }
            LexicalError::UnknownError => {
                Report::lexer_error(
                    Range { start: 0, end: 0 },
                    "Unknown error occurred".to_string(),
                )
            }
        }
    }
}

fn parser_error_to_report(error: &ParseError<usize, Token<'_>, LexicalError>) -> Report {
    match error {
        ParseError::InvalidToken { location } => {
            Report::parser_error(*location, *location, "Invalid token found".to_string())
        }
        ParseError::UnrecognizedEof { location, expected } => {
            let tokens = expected
                .iter()
                .fold(String::new(), |init, c| format!("{} {}", init, c))
                .trim()
                .to_string();
            let expected = if expected.is_empty() {
                String::new()
            } else {
                format!(" Expected: {}", tokens)
            };
            let message = format!("Unexpected end of file.{}", expected);
            Report::parser_error(*location, *location, message)
        }
        ParseError::UnrecognizedToken { token, expected } => {
            let tokens = expected
                .iter()
                .fold(String::new(), |init, c| format!("{} {}", init, c))
                .trim()
                .to_string();
            let expected = if expected.is_empty() {
                String::new()
            } else {
                format!(" Expected: {}", tokens)
            };
            let message = format!(
                "Unrecognised token, {}, at this location.{}",
                token.1, expected
            );
            Report::parser_error(token.0, token.2, message)
        }
        ParseError::ExtraToken { token } => {
            let message = format!("Unrecognised token, {}, at this location", token.1);
            Report::parser_error(token.0, token.2, message)
        }
        ParseError::User { error } => Report::from(error.clone()),
    }
}
