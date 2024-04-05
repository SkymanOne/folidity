use std::fmt::Display;

use folidity_diagnostics::Report;
use folidity_parser::{
    ast::Identifier,
    Span,
};

use crate::contract::ContractDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Struct,
    Model,
    State,
    Enum,
    Function,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum GlobalSymbol {
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),
    Function(SymbolInfo),
}

impl Default for GlobalSymbol {
    fn default() -> Self {
        GlobalSymbol::Function(Default::default())
    }
}

impl GlobalSymbol {
    /// Lookup symbol by ident in the contract definition,
    /// and add diagnostic error if not present.
    pub fn lookup(contract: &mut ContractDefinition, ident: &Identifier) -> Option<Self> {
        match contract.declaration_symbols.get(&ident.name) {
            Some(v) => Some(v.clone()),
            None => {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    String::from("Not declared."),
                ));
                None
            }
        }
    }

    /// Extract location.
    pub fn loc(&self) -> &Span {
        match self {
            GlobalSymbol::Struct(s) => &s.loc,
            GlobalSymbol::Model(s) => &s.loc,
            GlobalSymbol::Enum(s) => &s.loc,
            GlobalSymbol::State(s) => &s.loc,
            GlobalSymbol::Function(s) => &s.loc,
        }
    }

    /// Extract symbol info.
    pub fn symbol_info(&self) -> &SymbolInfo {
        match self {
            GlobalSymbol::Struct(s) => s,
            GlobalSymbol::Model(s) => s,
            GlobalSymbol::Enum(s) => s,
            GlobalSymbol::State(s) => s,
            GlobalSymbol::Function(s) => s,
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
#[derive(Debug, Clone, PartialEq, Default, Hash, Eq)]
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
