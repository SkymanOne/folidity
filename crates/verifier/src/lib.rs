use executor::SymbolicExecutor;
use folidity_semantics::{
    ContractDefinition,
    Runner,
};
use z3::{
    Config,
    Context,
};

mod ast;
mod executor;
mod resolver;

/// Create config for the Z3 context.
pub fn z3_cfg() -> Config {
    let mut cfg = Config::new();
    cfg.set_model_generation(true);
    // 10s timeout for constraint solving.
    cfg.set_timeout_msec(10_000);
    cfg
}

impl<'ctx> Runner<ContractDefinition, ()> for SymbolicExecutor<'ctx> {
    fn run(source: &ContractDefinition) -> Result<(), folidity_semantics::CompilationError>
    where
        Self: std::marker::Sized,
    {
        let context = Context::new(&z3_cfg());
        let executor = SymbolicExecutor::new(&context);

        Ok(())
    }
}
