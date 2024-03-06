use std::str::FromStr;

use algonaut_core::Address;
use folidity_diagnostics::Report;
use folidity_parser::{
    ast as parsed_ast,
    Span,
};

use crate::{
    ast::{
        Expression,
        TypeVariant,
        UnaryExpression,
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
    expression,
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
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    Ok(Expression::Boolean(UnaryExpression {
                        loc,
                        element: value,
                        ty: TypeVariant::Bool,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let expected = dynamic_to_concrete_type(tys, &[TypeVariant::Bool]);
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
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Char => {
                    Ok(Expression::Char(UnaryExpression {
                        loc,
                        element: value,
                        ty: TypeVariant::Char,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let expected = dynamic_to_concrete_type(tys, &[TypeVariant::Char]);
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
// TODO: support string formatting
pub fn resolve_string(
    value: String,
    loc: Span,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::String => {
                    Ok(Expression::String(UnaryExpression {
                        loc,
                        element: value,
                        ty: TypeVariant::String,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let expected = dynamic_to_concrete_type(tys, &[TypeVariant::String]);
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
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Hex => {
                    let bytes = hex::decode(value).map_err(|_| {
                        contract.diagnostics.push(Report::semantic_error(
                            loc.clone(),
                            String::from("Value is not a valid hex string."),
                        ));
                    })?;
                    Ok(Expression::Hex(UnaryExpression {
                        loc,
                        element: bytes,
                        ty: TypeVariant::Hex,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let expected = dynamic_to_concrete_type(tys, &[TypeVariant::Hex]);
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
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Address => {
                    let address = Address::from_str(value).map_err(|_| {})?;
                    Ok(Expression::Address(UnaryExpression {
                        loc,
                        element: address,
                        ty: TypeVariant::Address,
                    }))
                }
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            let expected = dynamic_to_concrete_type(tys, &[TypeVariant::Address]);
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

/// Resolve list and set of expression to a list of AST expressions.
///
/// # Notes
///
/// The list size is dynamic, so we don't check for sizes of any sub-arrays.
///
/// If the expected type is provided then we resolve every element to it,
/// and report error if any.
///
/// If the expected type is not provided, then deduce the type from the first element of
/// the list, and the resolve all consequent elements to that type.
///
/// # Errors
/// - The expected type is different.
/// - No expected types are provided and list contains no elements -> we can't deduce the
///   type.
/// - Elements are of different types.
pub fn resolve_lists(
    exprs: &[parsed_ast::Expression],
    loc: Span,
    contract: &mut ContractDefinition,
    scope: &mut Scope,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let mut derive_expr = |ty: &TypeVariant, loc: Span| -> Result<Expression, ()> {
        let mut error = false;
        let eval_exprs: Vec<Expression> = exprs
            .iter()
            .filter_map(|e| {
                if let Ok(e) = expression(e, ExpectedType::Concrete(ty.clone()), scope, contract) {
                    Some(e)
                } else {
                    error = true;
                    None
                }
            })
            .collect();

        if error {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("List elements appear to be of different types."),
            ));
            Err(())
        } else {
            Ok(Expression::List(UnaryExpression {
                loc,
                element: eval_exprs,
                ty: TypeVariant::List(Box::new(ty.clone())),
            }))
        }
    };
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Set(ty) => derive_expr(ty, loc),
                TypeVariant::List(ty) => derive_expr(ty, loc),
                a_ty => {
                    report_type_mismatch(&[expected_ty.clone()], a_ty, &loc, contract);
                    Err(())
                }
            }
        }
        ExpectedType::Dynamic(tys) => {
            if tys.is_empty() {
                // if there are no expected types, then we derive it from the first
                // element in the list.
                if exprs.is_empty() {
                    contract.diagnostics.push(Report::type_error(
                        loc,
                        String::from(
                            "Cannot derive type from the empty list without the type annotation.",
                        ),
                    ));
                    return Err(());
                }
                let expr = expression(&exprs[0], ExpectedType::Dynamic(vec![]), scope, contract)?;
                let exprt_ty =
                    ExpectedType::Concrete(TypeVariant::List(Box::new(expr.ty().clone())));
                resolve_lists(exprs, loc, contract, scope, exprt_ty)
            } else {
                // we need to manually inspect the type.
                let allowed_tys: Vec<TypeVariant> = tys
                    .iter()
                    .filter(|ty| matches!(ty, TypeVariant::List(_) | TypeVariant::Set(_)))
                    .cloned()
                    .collect();
                if allowed_tys.is_empty() {
                    contract.diagnostics.push(Report::semantic_error(
                        loc,
                        format!("Expected list or set, found {:?}", tys),
                    ));
                    return Err(());
                }
                let concrete = ExpectedType::Concrete(allowed_tys.first().unwrap().clone());
                resolve_lists(exprs, loc, contract, scope, concrete)
            }
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc,
                String::from("List literals can only be used in expressions."),
            ));
            Err(())
        }
    }
}
