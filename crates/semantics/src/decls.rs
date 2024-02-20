#[derive(Debug, Clone)]
pub struct DelayedDeclaration<T> {
    pub decl: T,
    pub i: usize,
}

/// Delayed declarations for the second pass semantic analysis.
#[derive(Debug, Default)]
pub struct DelayedDeclarations {
    pub structs: Vec<DelayedDeclaration<folidity_parser::ast::StructDeclaration>>,
    pub models: Vec<DelayedDeclaration<folidity_parser::ast::ModelDeclaration>>,
    pub states: Vec<DelayedDeclaration<folidity_parser::ast::StateDeclaration>>,
}
