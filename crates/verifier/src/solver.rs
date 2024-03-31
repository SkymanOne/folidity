use indexmap::IndexMap;
use z3::{
    ast::Bool,
    Context,
    SatResult,
    Solver,
};

use crate::{
    ast::Constraint,
    Diagnostics,
};

pub fn verify_constraints<'ctx>(
    constraints: &[&Constraint],
    context: &'ctx Context,
) -> Result<(), Vec<u32>> {
    let binding_consts: Vec<Bool<'ctx>> = constraints
        .iter()
        .map(|c| c.sym_to_const(context))
        .collect();

    let solver = Solver::new(context);
    for c in constraints {
        solver.assert(&c.expr);
    }

    match solver.check_assumptions(&binding_consts) {
        SatResult::Sat => Ok(()),
        SatResult::Unsat | SatResult::Unknown => {
            let consts = solver
                .get_unsat_core()
                .iter()
                .filter_map(|b| bool_const_to_id(b))
                .collect();
            Err(consts)
        }
    }
}

fn bool_const_to_id(c: &Bool) -> Option<u32> {
    c.to_string().replace("k!", "").parse().ok()
}
