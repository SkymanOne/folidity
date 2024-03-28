use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::ast::Expression;
use num_bigint::BigInt;
use num_rational::BigRational;
use z3::{
    ast::{
        Bool,
        Dynamic,
        Int,
        Real,
        String,
    },
    Context,
};

use crate::{
    ast::{
        Constraint,
        Z3BinaryExpression,
        Z3Expression,
        Z3UnaryExpression,
    },
    executor::SymbolicExecutor,
};

/// Transforms [`folidity_semantics::ast::Expression`] into [`crate::ast::Z3Expression`]
/// in some given context [`z3::Context`].
pub fn transform_expr<'ctx>(expr: &Expression, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    match expr {
        // literals
        Expression::Int(u) => int(&u.element, &u.loc, ctx),
        Expression::UInt(u) => int(&u.element.clone().into(), &u.loc, ctx),
        Expression::Float(u) => real(&u.element, &u.loc, ctx),
        Expression::Boolean(u) => bool(u.element, &u.loc, ctx),
        Expression::String(u) => string(u.element.as_str(), &u.loc, ctx),
        Expression::Char(u) => char(u.element, &u.loc, ctx),
        Expression::Hex(u) => string(hex::encode(&u.element).as_str(), &u.loc, ctx),
        Expression::Address(u) => string(u.element.to_string().as_str(), &u.loc, ctx),
        Expression::Enum(u) => enum_(u.element, &u.loc, ctx),

        // binary operations
        Expression::Multiply(_) => todo!(),
        Expression::Divide(_) => todo!(),
        Expression::Modulo(_) => todo!(),
        Expression::Add(_) => todo!(),
        Expression::Subtract(_) => todo!(),
        Expression::Equal(_) => todo!(),
        Expression::NotEqual(_) => todo!(),
        Expression::Greater(_) => todo!(),
        Expression::Less(_) => todo!(),
        Expression::GreaterEq(_) => todo!(),
        Expression::LessEq(_) => todo!(),
        Expression::Not(_) => todo!(),
        Expression::Or(_) => todo!(),
        Expression::And(_) => todo!(),
        Expression::In(_) => todo!(),

        Expression::MemberAccess(_) => todo!(),
        Expression::List(_) => todo!(),
        Expression::Variable(_) => todo!(),

        Expression::FunctionCall(_) => {
            todo!("Verification of function calls is not currently unsupported.")
        }
        Expression::StructInit(_) => {
            todo!("Verification of struct initialisation is not currently unsupported.")
        }
    }
}

fn int<'ctx>(value: &BigInt, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Int::from_big_int(ctx, value);
    Z3Expression::Int(Z3UnaryExpression {
        loc: loc.clone(),
        element: c,
    })
}

fn real<'ctx>(value: &BigRational, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Real::from_big_rational(ctx, value);
    Z3Expression::Real(Z3UnaryExpression {
        loc: loc.clone(),
        element: c,
    })
}

fn bool<'ctx>(value: bool, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Bool::from_bool(ctx, value);
    Z3Expression::Boolean(Z3UnaryExpression {
        loc: loc.clone(),
        element: c,
    })
}

fn string<'ctx>(value: &str, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = String::from_str(ctx, value).unwrap_or(String::fresh_const(ctx, ""));
    Z3Expression::String(Z3UnaryExpression {
        loc: loc.clone(),
        element: c,
    })
}

fn char<'ctx>(value: char, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Int::from_u64(ctx, value as u64);
    Z3Expression::Int(Z3UnaryExpression {
        loc: loc.clone(),
        element: c,
    })
}

fn enum_<'ctx>(value: usize, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Int::from_u64(ctx, value as u64);
    Z3Expression::Int(Z3UnaryExpression {
        loc: loc.clone(),
        element: c,
    })
}

/// Create a boolean constant and returns its id as `u32`
pub fn create_constraint_const<'ctx>(
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> (Bool<'ctx>, u32) {
    let bool_const = Bool::new_const(ctx, executor.symbol_counter);
    let id = executor.symbol_counter;
    executor.symbol_counter += 1;

    (bool_const, id)
}
