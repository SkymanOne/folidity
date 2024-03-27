use bounds::resolve_bounds;
pub use contract::ContractDefinition;
use folidity_diagnostics::Report;
use folidity_parser::ast::Source;
pub use folidity_parser::Span;
use functions::resolve_func_body;
pub use global_symbol::{
    GlobalSymbol,
    SymbolInfo,
};
use types::check_inheritance;

pub mod ast;
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

#[derive(Debug, Clone)]
pub enum CompilationError {
    Syntax(Vec<Report>),
    Formal(Vec<Report>),
    Emit(Vec<Report>),
}

impl CompilationError {
    pub fn diagnostics(&self) -> &Vec<Report> {
        match self {
            CompilationError::Syntax(r) => r,
            CompilationError::Formal(r) => r,
            CompilationError::Emit(r) => r,
        }
    }
}

/// Recursively walk the tree and modify the program artifacts.
pub trait Runner<S> {
    fn run(source: &S) -> Result<Self, CompilationError>
    where
        Self: std::marker::Sized;
}

impl Runner<Source> for ContractDefinition {
    fn run(source: &Source) -> Result<ContractDefinition, CompilationError> {
        let mut definition = ContractDefinition::default();
        definition.diagnostics.extend(source.diagnostics.clone());
        let mut delay = definition.resolve_declarations(source);
        definition.resolve_fields(&delay);

        check_inheritance(&mut definition, &delay);

        // todo: add built-in function to environment.

        // we can now resolve functions and create scopes.
        definition.resolve_functions(source, &mut delay);

        // now we can resolve model bounds on all declarations.
        resolve_bounds(&mut definition, &delay);

        for f in &delay.functions {
            let _ = resolve_func_body(&f.decl, f.i, &mut definition);
        }

        if !definition.diagnostics.is_empty() {
            return Err(CompilationError::Syntax(definition.diagnostics));
        }

        Ok(definition)
    }
}
