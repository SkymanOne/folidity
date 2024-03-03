use std::fmt::Display;

use folidity_diagnostics::Report;
use folidity_parser::{ast::Identifier, Span};

use crate::contract::ContractDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Struct,
    Model,
    State,
    Enum,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalSymbol {
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),
    Function(SymbolInfo),
}

impl GlobalSymbol {
    /// Lookup symbol by ident in the contract definition,
    /// and add diagnostic error if not present.
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

impl Display for GlobalSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut word = |s: &str| -> std::fmt::Result { write!(f, "{s}") };
        match self {
            GlobalSymbol::Struct(_) => word("struct"),
            GlobalSymbol::Model(_) => word("model"),
            GlobalSymbol::Enum(_) => word("enum"),
            GlobalSymbol::State(_) => word("state"),
            GlobalSymbol::Function(_) => word("function"),
        }
    }
}

impl Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut word = |s: &str| -> std::fmt::Result { write!(f, "{s}") };
        match self {
            SymbolKind::Struct => word("struct"),
            SymbolKind::Model => word("model"),
            SymbolKind::Enum => word("enum"),
            SymbolKind::State => word("state"),
            SymbolKind::Function => word("function"),
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
