use bounds::resolve_bounds;
use contract::ContractDefinition;
use folidity_parser::ast::Source;
use types::check_inheritance;

mod ast;
mod bounds;
mod contract;
mod expression;
mod functions;
mod global_symbol;
mod statement;
mod symtable;
mod types;

#[cfg(test)]
mod tests;

/// Resolves the contract's parsed tree into the semantically analysed and typed-checked
/// definition.
///
/// # Errors
/// [`ContractDefinition`] may contain errors stored in the `diagnostics` field.
pub fn resolve_semantics(source: &Source) -> ContractDefinition {
    let mut definition = ContractDefinition::default();
    let mut delay = definition.resolve_declarations(source);
    definition.resolve_fields(&delay);

    check_inheritance(&mut definition, &delay);

    // todo: add built-in function to environment.

    // we can now resolve functions and create scopes.
    definition.resolve_functions(source, &mut delay);

    // now we can resolve model bounds on all declarations.
    resolve_bounds(&mut definition, &delay);

    definition
}
