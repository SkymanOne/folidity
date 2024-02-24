#[derive(Debug, Clone)]
pub struct DelayedDeclaration<T> {
    pub decl: T,
    pub i: usize,
}

/// Saved declaration for later analysis.
/// The first pass should resolve the fields.
/// The second pass should resolve model bounds.
#[derive(Debug, Default)]
pub struct DelayedDeclarations {
    pub structs: Vec<DelayedDeclaration<folidity_parser::ast::StructDeclaration>>,
    pub models: Vec<DelayedDeclaration<folidity_parser::ast::ModelDeclaration>>,
    pub states: Vec<DelayedDeclaration<folidity_parser::ast::StateDeclaration>>,
}
