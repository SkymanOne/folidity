use folidity_diagnostics::Report;
use folidity_parser::{
    ast as parsed_ast,
    Span,
};

use crate::{
    ast::{
        BinaryExpression,
        Expression,
        TypeVariant,
    },
    contract::ContractDefinition,
    symtable::Scope,
    types::{
        report_type_mismatch,
        ExpectedType,
    },
};

use super::{
    dynamic_to_concrete_type,
    eval::eval_const,
    expression,
};

/// Resolve multiplication.
pub fn resolve_multiply(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[TypeVariant::Int, TypeVariant::Uint, TypeVariant::Float];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Int | TypeVariant::Uint | TypeVariant::Float => {
                    let resolved_left = expression(left, expected_ty.clone(), scope, contract);
                    let resolved_right = expression(right, expected_ty.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Multiply(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: ty.clone(),
                    });
                    if right.is_literal() && left.is_literal() {
                        eval_const(&expr, loc, contract)
                    } else {
                        Ok(expr)
                    }
                }
                a_ty => {
                    let expected: Vec<ExpectedType> = allowed_tys
                        .iter()
                        .map(|ty| ExpectedType::Concrete(ty.clone()))
                        .collect();
                    report_type_mismatch(expected.as_slice(), a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let concrete = deduce_type(left, right, &loc, tys, allowed_tys, scope, contract)?;
            resolve_multiply(left, right, loc, scope, contract, concrete)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Multiplication can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Find a valid concrete type from the list of allowed types.
/// - If suggested types are empty, we resolve the type from the left hand expression.
/// - Otherwise, we check every possible allowed type and filter out the ones to which the
///   left-hand expression can not be resolved.
/// or check the right one.
/// # Errors
/// - Expression can not be resolved to any of the allowed types.
fn deduce_type(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: &Span,
    tys: &Vec<TypeVariant>,
    allowed_tys: &[TypeVariant],
    scope: &mut Scope,
    contract: &mut ContractDefinition,
) -> Result<ExpectedType, ()> {
    if tys.is_empty() {
        let expr = expression(left, ExpectedType::Dynamic(vec![]), scope, contract)?;
        Ok(ExpectedType::Concrete(expr.ty().clone()))
    } else {
        // just clone the scope and contract definition as we need to dry run expression
        // resolution.
        // todo: optimise later
        let mut scope = scope.clone();
        let mut contract = contract.clone();
        // we find which types are allowed by checking whether the left hand side expression can
        // be resolved to it.
        let filtered_tys: Vec<TypeVariant> = allowed_tys
            .iter()
            .filter(|&ty| {
                expression(
                    left,
                    ExpectedType::Concrete(ty.clone()),
                    &mut scope,
                    &mut contract,
                )
                .is_ok()
                    || expression(
                        right,
                        ExpectedType::Concrete(ty.clone()),
                        &mut scope,
                        &mut contract,
                    )
                    .is_ok()
            })
            .cloned()
            .collect();

        if filtered_tys.is_empty() {
            contract.diagnostics.push(Report::type_error(
                loc.clone(),
                String::from("Cannot resolve these expression to any of the supported types."),
            ));
            return Err(());
        }
        Ok(dynamic_to_concrete_type(tys, allowed_tys))
    }
}
