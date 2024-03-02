use std::str::FromStr;

use algonaut::core::Address;
use folidity_diagnostics::Report;
use folidity_parser::{ast as parsed_ast, Span};

use crate::{
    ast::{Expression, TypeVariant},
    contract::ContractDefinition,
    types::{report_type_mismatch, ExpectedType},
};

/// Resolve bool to an expression.
///
/// # Errors
/// - Expected type is different.
pub fn resolve_bool(
    value: bool,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::Bool => Ok(Expression::Boolean(parsed_ast::UnaryExpression {
                loc,
                element: value,
            })),
            a_ty => {
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            let mut allowed_tys = tys.iter().filter(|ty| matches!(ty, TypeVariant::Bool));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::Bool)
            };
            resolve_bool(value, loc, contract, expected)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("Boolean literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}

/// Resolve char to an expression.
///
/// # Errors
/// - Expected type is different.
pub fn resolve_char(
    value: char,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::Char => Ok(Expression::Char(parsed_ast::UnaryExpression {
                loc,
                element: value,
            })),
            a_ty => {
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            let mut allowed_tys = tys.iter().filter(|ty| matches!(ty, TypeVariant::Char));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::Char)
            };
            resolve_char(value, loc, contract, expected)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("Char literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}

/// Resolve string to an expression.
///
/// # Errors
/// - Expected type is different.
//TODO: support string formatting
pub fn resolve_string(
    value: String,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::String => Ok(Expression::String(parsed_ast::UnaryExpression {
                loc,
                element: value,
            })),
            a_ty => {
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            let mut allowed_tys = tys.iter().filter(|ty| matches!(ty, TypeVariant::String));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::String)
            };
            resolve_string(value, loc, contract, expected)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("String literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}

/// Resolve string to a hex expression decoding it to bytes.
///
/// # Errors
/// - Provided string is not a valid hex.
/// - Expected type is different.
pub fn resolve_hex(
    value: &str,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::Hex => {
                let bytes = hex::decode(value).map_err(|_| {
                    contract.diagnostics.push(Report::semantic_error(
                        loc.clone(),
                        String::from("Value is not a valid hex string."),
                    ));
                })?;
                Ok(Expression::Hex(parsed_ast::UnaryExpression {
                    loc,
                    element: bytes,
                }))
            }
            a_ty => {
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            let mut allowed_tys = tys.iter().filter(|ty| matches!(ty, TypeVariant::Hex));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::Hex)
            };
            resolve_hex(value, loc, contract, expected)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("Hex literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}

/// Resolve string to an expression decoding it to a valid Algorand address.
///
/// # Errors
/// - Provided string is not a valid address.
/// - Expected type is different.
pub fn resolve_address(
    value: &str,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => match ty {
            TypeVariant::Address => {
                let address = Address::from_str(value).map_err(|_| {})?;
                Ok(Expression::Address(parsed_ast::UnaryExpression {
                    loc,
                    element: address,
                }))
            }
            a_ty => {
                report_type_mismatch(&expected_ty, a_ty, &loc, contract);
                Err(())
            }
        },
        ExpectedType::Dynamic(tys) => {
            let mut allowed_tys = tys.iter().filter(|ty| matches!(ty, TypeVariant::Address));
            let expected = if let Some(ty) = allowed_tys.next() {
                ExpectedType::Concrete(ty.clone())
            } else {
                ExpectedType::Concrete(TypeVariant::Address)
            };
            resolve_address(value, loc, contract, expected)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("Address literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}
