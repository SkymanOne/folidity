use z3::Config;

mod ast;
mod executor;

/// Create config for the Z3 context.
pub fn z3_cfg() -> Config {
    let mut cfg = Config::new();
    cfg.set_model_generation(true);
    // 10s timeout for constraint solving.
    cfg.set_timeout_msec(10_000);
    cfg
}
