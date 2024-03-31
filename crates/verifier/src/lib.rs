use executor::SymbolicExecutor;
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
mod transformer;

#[cfg(test)]
mod tests;

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

        if executor.resolve_declarations(source).is_err() {
            return Err(CompilationError::Formal(executor.diagnostics));
        }

        Ok(())
    }
}
