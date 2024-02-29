use folidity_diagnostics::Report;
use folidity_parser::{ast::Identifier, Span};

use crate::contract::ContractDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalSymbol {
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),
    Function(SymbolInfo),
}

impl GlobalSymbol {
    pub fn lookup<'a>(
        contract: &'a mut ContractDefinition,
        ident: &'a Identifier,
    ) -> Option<&'a Self> {
        match contract.declaration_symbols.get(&ident.name) {
            Some(v) => Some(v),
            None => {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    String::from("Not declared."),
                ));
                None
            }
        }
    }
}

/// Global user defined symbol info.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    /// Locations of the global symbol.
    pub loc: Span,
    /// Index of the global symbol.
    pub i: usize,
}

impl SymbolInfo {
    pub fn new(loc: Span, i: usize) -> Self {
        Self { loc, i }
    }
}
