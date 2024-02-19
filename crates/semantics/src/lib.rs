use contract::ContractDefinition;
use folidity_parser::ast::Source;

mod ast;
mod contract;
mod decls;
mod global_symbol;
mod symtable;
mod types;

#[cfg(test)]
mod tests;

/// Resolves the contract's parsed tree into the semantically analysed and typed-checked definition.
///
/// # Errors
/// [`ContractDefinition`] may contain errors stored in the `diagnostics` field.
pub fn resolve_semantics(source: &Source) -> ContractDefinition {
    let mut definition = ContractDefinition::default();
    let delay = definition.resolve_declarations(source);

    definition
}
