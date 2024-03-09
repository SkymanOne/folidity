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
    eval::eval_const,
    expression,
};

/// Resolve multiplication.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
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
            let concrete = coerce_type(left, right, &loc, tys, allowed_tys, scope, contract)?;
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

/// Resolve division.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_division(
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

                    let expr = Expression::Divide(BinaryExpression {
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
            let concrete = coerce_type(left, right, &loc, tys, allowed_tys, scope, contract)?;
            resolve_division(left, right, loc, scope, contract, concrete)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Division can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve modulo.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_modulo(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[TypeVariant::Int, TypeVariant::Uint];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Int | TypeVariant::Uint => {
                    let resolved_left = expression(left, expected_ty.clone(), scope, contract);
                    let resolved_right = expression(right, expected_ty.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Modulo(BinaryExpression {
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
            let concrete = coerce_type(left, right, &loc, tys, allowed_tys, scope, contract)?;
            resolve_modulo(left, right, loc, scope, contract, concrete)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Modulo can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve addition.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_addition(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::String,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Int | TypeVariant::Uint | TypeVariant::Float | TypeVariant::String => {
                    let resolved_left = expression(left, expected_ty.clone(), scope, contract);
                    let resolved_right = expression(right, expected_ty.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Add(BinaryExpression {
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
            let concrete = coerce_type(left, right, &loc, tys, allowed_tys, scope, contract)?;
            resolve_addition(left, right, loc, scope, contract, concrete)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Addition can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve subtraction.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_subtraction(
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

                    let expr = Expression::Subtract(BinaryExpression {
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
            let concrete = coerce_type(left, right, &loc, tys, allowed_tys, scope, contract)?;
            resolve_subtraction(left, right, loc, scope, contract, concrete)
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Subtraction can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve equality.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_equality(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::String,
        TypeVariant::Char,
        TypeVariant::Hex,
        TypeVariant::Address,
        TypeVariant::Bool,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let concrete =
                        coerce_type(left, right, &loc, &vec![], allowed_tys, scope, contract)?;

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Equal(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_equality(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Equality can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve inequality.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_inequality(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::String,
        TypeVariant::Char,
        TypeVariant::Hex,
        TypeVariant::Address,
        TypeVariant::Bool,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let concrete =
                        coerce_type(left, right, &loc, &vec![], allowed_tys, scope, contract)?;

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::NotEqual(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_inequality(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Inequality can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve greater comparison.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_greater(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::Char,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let concrete =
                        coerce_type(left, right, &loc, &vec![], allowed_tys, scope, contract)?;

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Greater(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_greater(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`Greater` comparison can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve less comparison.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_less(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::Char,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let concrete =
                        coerce_type(left, right, &loc, &vec![], allowed_tys, scope, contract)?;

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Less(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_less(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`Less` comparison can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve greater or equal comparison.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_greater_eq(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::Char,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let concrete =
                        coerce_type(left, right, &loc, &vec![], allowed_tys, scope, contract)?;

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::GreaterEq(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_greater_eq(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`GreaterEq` comparison can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve less or equal comparison.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_less_eq(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[
        TypeVariant::Int,
        TypeVariant::Uint,
        TypeVariant::Float,
        TypeVariant::Char,
    ];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let concrete =
                        coerce_type(left, right, &loc, &vec![], allowed_tys, scope, contract)?;

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::LessEq(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_less_eq(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`LessEq` comparison can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve boolean conjunction.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_and(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[TypeVariant::Bool];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    // we only allow boolean value to be resolved.
                    let concrete = ExpectedType::Concrete(TypeVariant::Bool);

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::And(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_and(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`And` operation can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve boolean disjunction.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_or(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[TypeVariant::Bool];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    // we only allow boolean value to be resolved.
                    let concrete = ExpectedType::Concrete(TypeVariant::Bool);

                    let resolved_left = expression(left, concrete.clone(), scope, contract);
                    let resolved_right = expression(right, concrete.clone(), scope, contract);

                    if resolved_left.is_err() || resolved_right.is_err() {
                        return Err(());
                    }

                    let right = Box::new(resolved_right.unwrap());
                    let left = Box::new(resolved_left.unwrap());

                    let expr = Expression::Or(BinaryExpression {
                        loc: loc.clone(),
                        left: left.clone(),
                        right: right.clone(),
                        ty: TypeVariant::Bool,
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_or(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`Or` operation can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve boolean negation.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_not(
    expr: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[TypeVariant::Bool];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    // we only allow boolean value to be resolved.
                    let concrete = ExpectedType::Concrete(TypeVariant::Bool);

                    let resolved = expression(expr, concrete.clone(), scope, contract);

                    if resolved.is_err() {
                        return Err(());
                    }

                    let value = Box::new(resolved.unwrap());

                    let expr = Expression::Not(UnaryExpression {
                        loc: loc.clone(),
                        element: value.clone(),
                        ty: TypeVariant::Bool,
                    });
                    if value.is_literal() {
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_not(
                expr,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`Negation` operation can only be used in expression."),
            ));
            Err(())
        }
    }
}

/// Resolve list inclusion.
///
/// # Errors
/// - Expected type is different.
/// - One of expression can not be resolved to any of the allowed types.
pub fn resolve_in(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let allowed_tys = &[TypeVariant::Bool];
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            match ty {
                TypeVariant::Bool => {
                    let resolved_right =
                        expression(right, ExpectedType::Dynamic(vec![]), scope, contract)?;

                    let right_list_ty = match resolved_right.ty() {
                        TypeVariant::List(ty) => ty.as_ref(),
                        TypeVariant::Set(ty) => ty.as_ref(),
                        _ => {
                            contract.diagnostics.push(Report::type_error(
                                loc,
                                String::from("Expected list-like type."),
                            ));
                            return Err(());
                        }
                    };
                    let resolved_left = expression(
                        left,
                        ExpectedType::Concrete(right_list_ty.clone()),
                        scope,
                        contract,
                    )?;

                    Ok(Expression::In(BinaryExpression {
                        loc,
                        left: Box::new(resolved_left),
                        right: Box::new(resolved_right),
                        ty: TypeVariant::Bool,
                    }))
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
        // we can only resolve to boolean value.
        ExpectedType::Dynamic(_) => {
            resolve_in(
                left,
                right,
                loc,
                scope,
                contract,
                ExpectedType::Concrete(TypeVariant::Bool),
            )
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`Or` operation can only be used in expression."),
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
fn coerce_type(
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
