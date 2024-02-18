use folidity_parser::Span;

#[derive(Debug, Clone)]
pub enum GlobalSymbol {
    Struct(SymbolInfo),
    Model(SymbolInfo),
    Enum(SymbolInfo),
    State(SymbolInfo),
    Function(SymbolInfo),
}

/// Global user defined symbol info.
#[derive(Debug, Clone)]
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
