use std::ops::Mul;

use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::ast::Expression;
use num_bigint::BigInt;
use num_rational::BigRational;
use z3::{
    ast::{
        Ast,
        Bool,
        Dynamic,
        Int,
        Real,
        String as Z3String,
    },
    Context,
};

use crate::{
    ast::{
        Constraint,
        Z3Expression,
    },
    executor::SymbolicExecutor,
};

/// Transforms [`folidity_semantics::ast::Expression`] into [`crate::ast::Z3Expression`]
/// in some given context [`z3::Context`].
pub fn transform_expr<'ctx>(
    expr: &Expression,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    match expr {
        // literals
        Expression::Int(u) => Ok(int(&u.element, &u.loc, ctx)),
        Expression::UInt(u) => Ok(int(&u.element.clone().into(), &u.loc, ctx)),
        Expression::Float(u) => Ok(real(&u.element, &u.loc, ctx)),
        Expression::Boolean(u) => Ok(bool(u.element, &u.loc, ctx)),
        Expression::String(u) => Ok(string(u.element.as_str(), &u.loc, ctx)),
        Expression::Char(u) => Ok(char(u.element, &u.loc, ctx)),
        Expression::Hex(u) => Ok(string(hex::encode(&u.element).as_str(), &u.loc, ctx)),
        Expression::Address(u) => Ok(string(u.element.to_string().as_str(), &u.loc, ctx)),
        Expression::Enum(u) => Ok(enum_(u.element, &u.loc, ctx)),

        // binary operations
        Expression::Add(_)
        | Expression::Subtract(_)
        | Expression::Multiply(_)
        | Expression::Divide(_)
        | Expression::Less(_)
        | Expression::LessEq(_)
        | Expression::Greater(_)
        | Expression::GreaterEq(_) => int_real_op(expr, ctx, executor),

        Expression::Modulo(b) => modulo(&b.left, &b.right, &b.loc, ctx, executor),
        Expression::Equal(b) => equality(&b.left, &b.right, &b.loc, ctx, executor),
        Expression::NotEqual(b) => inequality(&b.left, &b.right, &b.loc, ctx, executor),
        Expression::Not(u) => not(&u.element, &u.loc, ctx, executor),

        Expression::Or(b) => or(&b.left, &b.right, &b.loc, ctx, executor),
        Expression::And(b) => and(&b.left, &b.right, &b.loc, ctx, executor),

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

fn int_real_op<'ctx>(
    e: &Expression,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let (Expression::Multiply(b)
    | Expression::Divide(b)
    | Expression::Add(b)
    | Expression::Subtract(b)
    | Expression::Less(b)
    | Expression::LessEq(b)
    | Expression::Greater(b)
    | Expression::GreaterEq(b)) = e
    else {
        unreachable!("Only [*, /, +, -, >, <, ≥, ≤] ops are allowed to be passed.");
    };
    let e1 = transform_expr(&b.left, ctx, executor)?;
    let e2 = transform_expr(&b.right, ctx, executor)?;
    let mut reports = Vec::new();
    let int1 = to_z3_int(&e1, &mut reports);
    let int2 = to_z3_int(&e2, &mut reports);
    let real1 = to_z3_real(&e1, &mut reports);
    let real2 = to_z3_real(&e2, &mut reports);
    let res = match (int1, int2, real1, real2) {
        (Ok(n1), Ok(n2), _, _) => {
            match e {
                Expression::Add(_) => Dynamic::from_ast(&(n1 + n2)),
                Expression::Subtract(_) => Dynamic::from_ast(&(n1 - n2)),
                Expression::Multiply(_) => Dynamic::from_ast(&(n1 * n2)),
                Expression::Divide(_) => Dynamic::from_ast(&(n1 / n2)),
                Expression::Less(_) => Dynamic::from_ast(&n1.lt(&n1)),
                Expression::LessEq(_) => Dynamic::from_ast(&n1.le(&n1)),
                Expression::Greater(_) => Dynamic::from_ast(&n1.gt(&n1)),
                Expression::GreaterEq(_) => Dynamic::from_ast(&n1.ge(&n1)),
                _ => unreachable!(),
            }
        }
        (_, _, Ok(n1), Ok(n2)) => {
            match e {
                Expression::Add(_) => Dynamic::from_ast(&(n1 + n2)),
                Expression::Subtract(_) => Dynamic::from_ast(&(n1 - n2)),
                Expression::Multiply(_) => Dynamic::from_ast(&(n1 * n2)),
                Expression::Divide(_) => Dynamic::from_ast(&(n1 / n2)),
                Expression::Less(_) => Dynamic::from_ast(&n1.lt(&n1)),
                Expression::LessEq(_) => Dynamic::from_ast(&n1.le(&n1)),
                Expression::Greater(_) => Dynamic::from_ast(&n1.gt(&n1)),
                Expression::GreaterEq(_) => Dynamic::from_ast(&n1.ge(&n1)),
                _ => unreachable!(),
            }
        }
        _ => {
            executor.diagnostics.push(Report::ver_error_with_extra(
                b.loc.clone(),
                String::from("Can not apply arithmetic operation on these data ."),
                reports,
            ));
            return Err(());
        }
    };
    Ok(Z3Expression {
        loc: b.loc.clone(),
        element: res,
    })
}

fn modulo<'ctx>(
    left: &Expression,
    right: &Expression,
    loc: &Span,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(left, ctx, executor)?;
    let e2 = transform_expr(right, ctx, executor)?;

    let mut reports = Vec::new();
    let int1 = to_z3_int(&e1, &mut reports);
    let int2 = to_z3_int(&e2, &mut reports);
    match (int1, int2) {
        (Ok(n1), Ok(n2)) => {
            let res = n1 % n2;
            Ok(Z3Expression::new(loc, &res))
        }
        _ => {
            executor.diagnostics.push(Report::ver_error_with_extra(
                loc.clone(),
                String::from("Can not perform modulo operation."),
                reports,
            ));
            Err(())
        }
    }
}

fn equality<'ctx>(
    left: &Expression,
    right: &Expression,
    loc: &Span,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(left, ctx, executor)?;
    let e2 = transform_expr(right, ctx, executor)?;

    let res = e1.element._safe_eq(&e2.element).map_err(|_| {
        executor.diagnostics.push(Report::ver_error(
            loc.clone(),
            String::from("Sort mismatch."),
        ))
    })?;

    Ok(Z3Expression::new(loc, &res))
}

fn inequality<'ctx>(
    left: &Expression,
    right: &Expression,
    loc: &Span,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(left, ctx, executor)?;
    let e2 = transform_expr(right, ctx, executor)?;

    let res = Dynamic::distinct(ctx, &[&e1.element, &e2.element]);

    Ok(Z3Expression::new(loc, &res))
}

fn not<'ctx>(
    e: &Expression,
    loc: &Span,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let v = transform_expr(e, ctx, executor)?;

    let bool_v = to_z3_bool(&v, &mut executor.diagnostics)?;

    Ok(Z3Expression::new(loc, &bool_v))
}

fn or<'ctx>(
    left: &Expression,
    right: &Expression,
    loc: &Span,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(left, ctx, executor)?;
    let e2 = transform_expr(right, ctx, executor)?;

    let mut reports = Vec::new();
    let int1 = to_z3_bool(&e1, &mut reports);
    let int2 = to_z3_bool(&e2, &mut reports);
    match (int1, int2) {
        (Ok(n1), Ok(n2)) => {
            let res = Bool::or(ctx, &[n1, n2]);
            Ok(Z3Expression::new(loc, &res))
        }
        _ => {
            executor.diagnostics.push(Report::ver_error_with_extra(
                loc.clone(),
                String::from("Can not perform boolean OR."),
                reports,
            ));
            Err(())
        }
    }
}

fn and<'ctx>(
    left: &Expression,
    right: &Expression,
    loc: &Span,
    ctx: &'ctx Context,
    executor: &mut SymbolicExecutor,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(left, ctx, executor)?;
    let e2 = transform_expr(right, ctx, executor)?;

    let mut reports = Vec::new();
    let int1 = to_z3_bool(&e1, &mut reports);
    let int2 = to_z3_bool(&e2, &mut reports);
    match (int1, int2) {
        (Ok(n1), Ok(n2)) => {
            let res = Bool::and(ctx, &[n1, n2]);
            Ok(Z3Expression::new(loc, &res))
        }
        _ => {
            executor.diagnostics.push(Report::ver_error_with_extra(
                loc.clone(),
                String::from("Can not perform boolean AND."),
                reports,
            ));
            Err(())
        }
    }
}

fn int<'ctx>(value: &BigInt, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Int::from_big_int(ctx, value);
    Z3Expression::new(loc, &c)
}

fn real<'ctx>(value: &BigRational, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Real::from_big_rational(ctx, value);
    Z3Expression::new(loc, &c)
}

fn bool<'ctx>(value: bool, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Bool::from_bool(ctx, value);
    Z3Expression::new(loc, &c)
}

fn string<'ctx>(value: &str, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Z3String::from_str(ctx, value).unwrap_or(Z3String::fresh_const(ctx, ""));
    Z3Expression::new(loc, &c)
}

fn char<'ctx>(value: char, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Int::from_u64(ctx, value as u64);
    Z3Expression::new(loc, &c)
}

fn enum_<'ctx>(value: usize, loc: &Span, ctx: &'ctx Context) -> Z3Expression<'ctx> {
    let c = Int::from_u64(ctx, value as u64);
    Z3Expression::new(loc, &c)
}

fn to_z3_int<'ctx>(
    expr: &Z3Expression<'ctx>,
    diagnostics: &mut Vec<Report>,
) -> Result<Int<'ctx>, ()> {
    expr.element.as_int().ok_or_else(|| {
        diagnostics.push(Report::ver_error(
            expr.loc.clone(),
            String::from("Value can not be converted to integer."),
        ));
    })
}

fn to_z3_real<'ctx>(
    expr: &Z3Expression<'ctx>,
    diagnostics: &mut Vec<Report>,
) -> Result<Real<'ctx>, ()> {
    expr.element.as_real().ok_or_else(|| {
        diagnostics.push(Report::ver_error(
            expr.loc.clone(),
            String::from("Value can not be converted to real."),
        ));
    })
}

fn to_z3_bool<'ctx>(
    expr: &Z3Expression<'ctx>,
    diagnostics: &mut Vec<Report>,
) -> Result<Bool<'ctx>, ()> {
    expr.element.as_bool().ok_or_else(|| {
        diagnostics.push(Report::ver_error(
            expr.loc.clone(),
            String::from("Value can not be converted to boolean."),
        ));
    })
}

fn to_z3_string<'ctx>(
    expr: &Z3Expression<'ctx>,
    diagnostics: &mut Vec<Report>,
) -> Result<Z3String<'ctx>, ()> {
    expr.element.as_string().ok_or_else(|| {
        diagnostics.push(Report::ver_error(
            expr.loc.clone(),
            String::from("Value can not be converted to string."),
        ));
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