//! Evaluate binary expression of two literals to a single literal.

use algonaut_core::Address;
use folidity_diagnostics::Report;
use folidity_parser::Span;
use num_bigint::{
    BigInt,
    BigUint,
};
use num_rational::BigRational;
use num_traits::{
    ops::checked::CheckedAdd,
    CheckedDiv,
    CheckedEuclid,
    CheckedMul,
    CheckedSub,
};

use crate::{
    ast::{
        BinaryExpression,
        Expression,
        TryGetValue,
        TypeVariant,
        UnaryExpression,
    },
    contract::ContractDefinition,
};

/// Evaluate constant expression to a literal value.
/// It assumes that type checking has been done correctly.
///
/// # Errors
/// - Division by 0
/// - Overflow
/// - Underflow
/// - Invalid types
pub fn eval_const(
    expr: &Expression,
    loc: Span,
    contract: &mut ContractDefinition,
) -> Result<Expression, ()> {
    match expr {
        Expression::Multiply(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Int(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Int,
                        |a, b| a.checked_mul(&b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::UInt(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Uint,
                        |a, b| a.checked_mul(&b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Float(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Float,
                        |a, b| a.checked_mul(&b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Divide(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Int(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Int,
                        |a, b| a.checked_div(&b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::UInt(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Uint,
                        |a, b| a.checked_div(&b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Float(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Float,
                        |a, b| a.checked_div(&b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Modulo(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Int(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Int,
                        |a, b| a.checked_rem_euclid(&b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::UInt(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Uint,
                        |a, b| a.checked_rem_euclid(&b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Add(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Int(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Int,
                        |a, b| a.checked_add(&b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::UInt(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Uint,
                        |a, b| a.checked_add(&b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Float(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Float,
                        |a, b| a.checked_add(&b),
                        contract,
                    )?))
                }
                TypeVariant::String => {
                    Ok(Expression::String(calc::<String, _, _>(
                        u,
                        loc,
                        TypeVariant::String,
                        |a, b| Some(format!("{}{}", a, b)),
                        contract,
                    )?))
                }

                _ => Err(()),
            }
        }
        Expression::Subtract(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Int(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Int,
                        |a, b| a.checked_sub(&b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::UInt(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Uint,
                        |a, b| a.checked_sub(&b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Float(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Float,
                        |a, b| a.checked_sub(&b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Equal(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Boolean(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::Boolean(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Boolean(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::String => {
                    Ok(Expression::Boolean(calc::<String, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::Char => {
                    Ok(Expression::Boolean(calc::<char, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::Bool => {
                    Ok(Expression::Boolean(calc::<bool, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::Hex => {
                    Ok(Expression::Boolean(calc::<Vec<u8>, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                TypeVariant::Address => {
                    Ok(Expression::Boolean(calc::<Address, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a == b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::NotEqual(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Boolean(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::Boolean(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Boolean(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::String => {
                    Ok(Expression::Boolean(calc::<String, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::Char => {
                    Ok(Expression::Boolean(calc::<char, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::Bool => {
                    Ok(Expression::Boolean(calc::<bool, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::Hex => {
                    Ok(Expression::Boolean(calc::<Vec<u8>, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                TypeVariant::Address => {
                    Ok(Expression::Boolean(calc::<Address, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a != b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Greater(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Boolean(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a > b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::Boolean(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a > b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Boolean(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a > b),
                        contract,
                    )?))
                }
                TypeVariant::Char => {
                    Ok(Expression::Boolean(calc::<char, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a > b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Less(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Boolean(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a < b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::Boolean(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a < b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Boolean(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a < b),
                        contract,
                    )?))
                }
                TypeVariant::Char => {
                    Ok(Expression::Boolean(calc::<char, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a < b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::GreaterEq(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Boolean(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a >= b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::Boolean(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a >= b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Boolean(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a >= b),
                        contract,
                    )?))
                }
                TypeVariant::Char => {
                    Ok(Expression::Boolean(calc::<char, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a >= b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::LessEq(u) => {
            match u.ty {
                TypeVariant::Int => {
                    Ok(Expression::Boolean(calc::<BigInt, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a <= b),
                        contract,
                    )?))
                }
                TypeVariant::Uint => {
                    Ok(Expression::Boolean(calc::<BigUint, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a <= b),
                        contract,
                    )?))
                }
                TypeVariant::Float => {
                    Ok(Expression::Boolean(calc::<BigRational, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a <= b),
                        contract,
                    )?))
                }
                TypeVariant::Char => {
                    Ok(Expression::Boolean(calc::<char, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a <= b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Or(u) => {
            match u.ty {
                TypeVariant::Bool => {
                    Ok(Expression::Boolean(calc::<bool, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a || b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::And(u) => {
            match u.ty {
                TypeVariant::Bool => {
                    Ok(Expression::Boolean(calc::<bool, _, _>(
                        u,
                        loc,
                        TypeVariant::Bool,
                        |a, b| Some(a && b),
                        contract,
                    )?))
                }
                _ => Err(()),
            }
        }
        Expression::Not(u) => {
            let value = !TryGetValue::<bool>::try_get(u.element.as_ref())?;
            Ok(Expression::Boolean({
                UnaryExpression {
                    loc,
                    element: value,
                    ty: TypeVariant::Bool,
                }
            }))
        }
        _ => {
            contract.diagnostics.push(Report::type_error(
                loc.clone(),
                String::from("This expression cannot be evaluated to a literal."),
            ));
            Err(())
        }
    }
}

fn calc<T, U, F>(
    u: &BinaryExpression,
    loc: Span,
    ty: TypeVariant,
    func: F,
    contract: &mut ContractDefinition,
) -> Result<UnaryExpression<U>, ()>
where
    F: Fn(T, T) -> Option<U>,
    Expression: TryGetValue<T>,
{
    let a = TryGetValue::<T>::try_get(u.left.as_ref())?;
    let b = TryGetValue::<T>::try_get(u.right.as_ref())?;
    let result = func(a, b).ok_or_else(|| {
        contract.diagnostics.push(Report::func_error(
            loc.clone(),
            String::from("The operation is invalid. Probably resulted from the division by 0 or overflow/underflow."),
        ));
    })?;
    Ok(UnaryExpression {
        loc,
        element: result,
        ty,
    })
}
