pub use executor::SymbolicExecutor;
use folidity_diagnostics::Report;
use folidity_semantics::{
    CompilationError,
    ContractDefinition,
    Runner,
};
use z3::{
    Config,
    Context,
};

mod ast;
mod executor;
mod links;
mod solver;
mod transformer;

#[cfg(test)]
mod tests;

type Diagnostics = Vec<Report>;

/// Create config for the Z3 context.
pub fn z3_cfg() -> Config {
    let mut cfg = Config::new();
    cfg.set_model_generation(true);
    // 10s timeout for constraint solving.
    cfg.set_timeout_msec(10_000);
    cfg
}

impl<'ctx> Runner<ContractDefinition, ()> for SymbolicExecutor<'ctx> {
    fn run(source: &ContractDefinition) -> Result<(), CompilationError>
    where
        Self: std::marker::Sized,
    {
        let context = Context::new(&z3_cfg());

        let mut executor = SymbolicExecutor::new(&context);

        let mut err = false;
        let delays = executor.resolve_declarations(source);
        executor.resolve_links(delays, source);

        err |= !executor.resolve_bounds(source);

        err |= !executor.verify_individual_blocks(source);

        // report errors in individual blocks earlier to avoid catching them in linked blocks.
        if err {
            return Err(CompilationError::Formal(executor.diagnostics));
        }

        err = !executor.verify_linked_blocks(source);
        if err {
            return Err(CompilationError::Formal(executor.diagnostics));
        }

        Ok(())
    }
}
