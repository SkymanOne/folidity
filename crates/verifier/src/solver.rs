use folidity_semantics::GlobalSymbol;
use z3::{
    ast::Bool,
    Context,
    SatResult,
    Solver,
};

use crate::ast::Constraint;

/// Verify the slice of constraints for satisfiability.
///
/// # Errors
/// - List of ids of constraints that contradict each other.
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

    let res = match solver.check_assumptions(&binding_consts) {
        SatResult::Sat => Ok(()),
        SatResult::Unsat | SatResult::Unknown => {
            let consts = solver
                .get_unsat_core()
                .iter()
                .filter_map(|b| bool_const_to_id(b))
                .collect();
            Err(consts)
        }
    };
    solver.reset();
    res
}

/// Verify the slice of constraints block for satisfiability.
///
/// # Errors
/// - List of mapping from symbol of declaration to the vector of contradicting constant
///   ids.
pub fn verify_constraint_blocks<'ctx>(
    constraints: &[(Constraint<'ctx>, GlobalSymbol)],
    context: &'ctx Context,
) -> Result<(), Vec<(u32, GlobalSymbol)>> {
    let binding_consts: Vec<Bool<'ctx>> = constraints
        .iter()
        .map(|c| c.0.sym_to_const(context))
        .collect();

    let solver = Solver::new(context);
    for c in constraints {
        solver.assert(&c.0.expr);
    }

    let res = match solver.check_assumptions(&binding_consts) {
        SatResult::Sat => Ok(()),
        SatResult::Unsat | SatResult::Unknown => {
            let consts: Vec<u32> = solver
                .get_unsat_core()
                .iter()
                .filter_map(|b| bool_const_to_id(b))
                .collect();

            let mut consts_syms: Vec<(u32, GlobalSymbol)> = constraints
                .iter()
                .filter_map(|(c, g)| {
                    if consts.contains(&c.binding_sym) {
                        Some((c.binding_sym, g.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            consts_syms.sort_by_key(|x| x.0);
            Err(consts_syms)
        }
    };
    solver.reset();
    res
}

/// Z3 converts integer names to `k!_` format, we need to parse it back to integers.
fn bool_const_to_id(c: &Bool) -> Option<u32> {
    c.to_string().replace("k!", "").parse().ok()
}
