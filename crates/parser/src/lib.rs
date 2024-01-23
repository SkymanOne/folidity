pub mod ast;
pub mod lexer;

use lalrpop_util::lalrpop_mod;
use std::ops::Range;

pub type Span = Range<usize>;

lalrpop_mod!(pub folidity);

#[cfg(test)]
mod tests;
