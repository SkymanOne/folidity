use std::str::FromStr;

use folidity_diagnostics::Report;
use folidity_parser::{ast as parsed_ast, Span};
use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;

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
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            // use first type to resolve in the list of possible types,
            // otherwise we resolve to signed int
            // The latter can happen when we have var declaration without the type annotation.
            let mut allowed_tys = tys
                .iter()
                .filter(|ty| matches!(ty, TypeVariant::Int | TypeVariant::Uint));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::Int)
            };
            resolve_integer(number_str, loc, contract, expected)
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

/// Resolve real number to an expression.
///
/// # Errors
/// - Expected type is different.
/// - Parsing real-number failed.
pub fn resolve_float(
    number_str: &str,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::Float => {
                let number_str = if number_str.starts_with('.') {
                    format!("0.{number_str}")
                } else {
                    number_str.to_string()
                };
                let number = BigRational::from_str(&number_str).map_err(|_| {
                    contract.diagnostics.push(Report::semantic_error(
                        loc.clone(),
                        String::from("Error parsing real number"),
                    ));
                })?;
                Ok(Expression::Float(parsed_ast::UnaryExpression {
                    loc,
                    element: number,
                }))
            }
            a_ty => {
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            let mut allowed_tys = tys.iter().filter(|ty| matches!(ty, TypeVariant::Float));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::Float)
            };
            resolve_float(number_str, loc, contract, expected)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("Float literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}
