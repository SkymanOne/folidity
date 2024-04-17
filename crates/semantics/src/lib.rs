use bounds::resolve_bounds;
pub use contract::ContractDefinition;
use folidity_diagnostics::Report;
use folidity_parser::ast::Source;
pub use folidity_parser::{
    ast::Identifier,
    Span,
};
use functions::resolve_func_body;
pub use global_symbol::{
    GlobalSymbol,
    SymbolInfo,
    SymbolKind,
};
pub use types::DelayedDeclaration;

use types::check_inheritance;

pub mod ast;
mod bounds;
mod contract;
mod expression;
mod functions;
mod global_symbol;
mod statement;
pub mod symtable;
mod types;

#[cfg(test)]
mod tests;

/// Pipeline specific error during compilation.
#[derive(Debug, Clone)]
pub enum CompilationError {
    /// Error occurred during parsing and/or semantic analysis.
    Syntax(Vec<Report>),
    /// Error ocurred during formal verification stage.
    Formal(Vec<Report>),
    /// Error occurred during code emission.
    Emit(Vec<Report>),
}

impl CompilationError {
    /// Extract the list of reports from error variant.
    pub fn diagnostics(&self) -> &Vec<Report> {
        match self {
            CompilationError::Syntax(r) => r,
            CompilationError::Formal(r) => r,
            CompilationError::Emit(r) => r,
        }
    }
}

/// Program runner that performs some operations on the input and output artifacts.
///
/// # Generic params
/// - `I` - input type
/// - `O` output type.
///
/// # Errors
/// [`CompilationError`] variant specific to the pipeline step.
pub trait Runner<I, O> {
    fn run(source: &I) -> Result<O, CompilationError>
    where
        Self: std::marker::Sized;
}

impl Runner<Source, ContractDefinition> for ContractDefinition {
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
