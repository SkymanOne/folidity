use std::str::FromStr;

use folidity_diagnostics::Report;
use folidity_parser::{ast as parsed_ast, Span};
use num_bigint::{BigInt, BigUint};

use crate::{
    ast::{Expression, TypeVariant},
    contract::ContractDefinition,
    types::{report_type_mismatch, ExpectedType},
};

/// Resolve signed and unsigned integers.
///
/// # Errors
/// - Expected type is different.
/// - Unsigned integer is provided, when signed one is expected
pub fn resolve_integer(
    number_str: &str,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::Int => {
                let number = BigInt::from_str(number_str).unwrap();
                Ok(Expression::Int(parsed_ast::UnaryExpression {
                    loc,
                    element: number,
                }))
            }
            TypeVariant::Uint => {
                let number = BigUint::from_str(number_str).map_err(|_| {
                    contract.diagnostics.push(Report::semantic_error(
                        loc.clone(),
                        String::from("Expected unsigned integer, got signed one"),
                    ));
                })?;
                Ok(Expression::UInt(parsed_ast::UnaryExpression {
                    loc,
                    element: number,
                }))
            }
            a_ty => {
                report_type_mismatch(&expected_ty, &a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            // here we don't care what other types are, we just resolve to the first one.
            let ty = tys.first().unwrap();
            resolve_integer(
                number_str,
                loc,
                contract,
                ExpectedType::Concrete(ty.clone()),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("Integer literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}
