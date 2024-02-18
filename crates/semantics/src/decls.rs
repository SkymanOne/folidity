#[derive(Debug, Clone)]
pub struct DelayedFields<T> {
    pub decl: T,
    pub i: usize,
}

/// Delayed declarations for the second pass semantic analysis.
#[derive(Debug, Default)]
pub struct DelayedDeclarations {
    pub structs: Vec<DelayedFields<folidity_parser::ast::StructDeclaration>>,
    pub models: Vec<DelayedFields<folidity_parser::ast::ModelDeclaration>>,
    pub states: Vec<DelayedFields<folidity_parser::ast::StateDeclaration>>,
}
