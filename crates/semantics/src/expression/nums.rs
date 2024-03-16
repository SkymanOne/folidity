use std::str::FromStr;

use folidity_diagnostics::Report;
use folidity_parser::Span;
use num_bigint::{
    BigInt,
    BigUint,
};
use num_rational::BigRational;

use crate::{
    ast::{
        Expression,
        TypeVariant,
        UnaryExpression,
    },
    contract::ContractDefinition,
    types::{
        report_type_mismatch,
        ExpectedType,
    },
};

use super::dynamic_to_concrete_type;

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
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Int => {
                    let number = BigInt::from_str(number_str).unwrap();
                    Ok(Expression::Int(UnaryExpression {
                        loc,
                        element: number,
                        ty: TypeVariant::Int,
                    }))
                }
                TypeVariant::Uint => {
                    let number = BigUint::from_str(number_str).map_err(|_| {
                        contract.diagnostics.push(Report::semantic_error(
                            loc.clone(),
                            String::from("Expected unsigned integer, got signed one"),
                        ));
                    })?;
                    Ok(Expression::UInt(UnaryExpression {
                        loc,
                        element: number,
                        ty: TypeVariant::Uint,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            // use first type to resolve in the list of possible types,
            // otherwise we resolve to signed int
            // The latter can happen when we have var declaration without the type
            // annotation.
            let allowed = [TypeVariant::Int, TypeVariant::Uint];
            match resolve_expected_type(&allowed, tys) {
                Ok(expected) => resolve_integer(number_str, loc, contract, expected),
                Err(_) => {
                    contract.diagnostics.push(Report::type_error(
                        loc,
                        String::from("Could not imply type for this number."),
                    ));
                    Err(())
                }
            }
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
        ExpectedType::Concrete(ty) => {
            match ty {
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
                    Ok(Expression::Float(UnaryExpression {
                        loc,
                        element: number,
                        ty: TypeVariant::Float,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let allowed = [TypeVariant::Float];
            match resolve_expected_type(&allowed, tys) {
                Ok(expected) => resolve_integer(number_str, loc, contract, expected),
                Err(_) => {
                    contract.diagnostics.push(Report::type_error(
                        loc,
                        String::from("Could not imply type for this number."),
                    ));
                    Err(())
                }
            }
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

fn resolve_expected_type(allowed: &[TypeVariant], tys: &[TypeVariant]) -> Result<ExpectedType, ()> {
    let expected = if tys.is_empty() {
        dynamic_to_concrete_type(&[], allowed)
    } else {
        let filtered: Vec<TypeVariant> = allowed
            .iter()
            .filter(|t| tys.contains(t))
            .cloned()
            .collect();
        if filtered.is_empty() {
            return Err(());
        }
        dynamic_to_concrete_type(&filtered, allowed)
    };
    Ok(expected)
}
