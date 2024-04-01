use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::{
    ast::{
        BinaryExpression,
        Expression,
        MemberAccess,
        TypeVariant,
        UnaryExpression,
    },
    symtable::Scope,
    ContractDefinition,
    GlobalSymbol,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use z3::{
    ast::{
        Ast,
        Bool,
        Dynamic,
        Int,
        Real,
        Set,
        String as Z3String,
    },
    Context,
    Sort,
};

use crate::{
    ast::{
        Z3Expression,
        Z3Scope,
    },
    executor::SymbolicExecutor,
    Diagnostics,
};

#[derive(Debug)]
pub struct TransformParams<'ctx, 'a> {
    pub ctx: &'ctx Context,
    pub z3_scope: &'a mut Z3Scope,
    pub scope: &'a Scope,
    pub contract: &'a ContractDefinition,
    pub diagnostics: &'a mut Diagnostics,
    pub executor: &'a mut SymbolicExecutor<'ctx>,
}

/// Transforms [`folidity_semantics::ast::Expression`] into [`crate::ast::Z3Expression`]
/// in some given context [`z3::Context`].
pub fn transform_expr<'ctx>(
    expr: &Expression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    match expr {
        // literals
        Expression::Int(u) => Ok(int(&u.element, &u.loc, &params.ctx)),
        Expression::UInt(u) => Ok(int(&u.element.clone().into(), &u.loc, &params.ctx)),
        Expression::Float(u) => Ok(real(&u.element, &u.loc, &params.ctx)),
        Expression::Boolean(u) => Ok(bool(u.element, &u.loc, &params.ctx)),
        Expression::String(u) => Ok(string(u.element.as_str(), &u.loc, &params.ctx)),
        Expression::Char(u) => Ok(char(u.element, &u.loc, &params.ctx)),
        Expression::Hex(u) => {
            Ok(string(
                hex::encode(&u.element).as_str(),
                &u.loc,
                &params.ctx,
            ))
        }
        Expression::Address(u) => Ok(string(u.element.to_string().as_str(), &u.loc, &params.ctx)),
        Expression::Enum(u) => Ok(enum_(u, params)),

        // binary operations
        Expression::Add(_)
        | Expression::Subtract(_)
        | Expression::Multiply(_)
        | Expression::Divide(_)
        | Expression::Less(_)
        | Expression::LessEq(_)
        | Expression::Greater(_)
        | Expression::GreaterEq(_) => int_real_op(expr, params),

        Expression::Modulo(b) => modulo(b, params),
        Expression::Equal(b) => equality(b, params),
        Expression::NotEqual(b) => inequality(b, params),
        Expression::Not(u) => not(u, params),

        Expression::Or(b) => or(b, params),
        Expression::And(b) => and(b, params),

        Expression::Variable(u) => variable(u, params),
        Expression::MemberAccess(m) => member_access(m, params),
        Expression::List(u) => list(u, params),
        Expression::In(b) => in_(b, params),

        Expression::FunctionCall(_) => {
            todo!("Verification of function calls is currently unsupported.")
        }
        Expression::StructInit(_) => {
            todo!("Verification of struct initialisation is currently unsupported.")
        }
    }
}

fn in_<'ctx>(
    b: &BinaryExpression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;
    let set = e2.element.as_set().ok_or_else(|| {
        params.diagnostics.push(Report::ver_error(
            b.right.loc().clone(),
            String::from("Expression can not be coerces to a Z3 `Set`"),
        ));
    })?;

    let assertion = set.member(&e1.element);

    Ok(Z3Expression::new(&b.loc, &assertion))
}

fn list<'ctx>(
    u: &UnaryExpression<Vec<Expression>>,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let mut set = Set::empty(&params.ctx, &type_to_sort(&u.ty, &params.ctx));
    for e in &u.element {
        let z3_e = transform_expr(e, params)?;
        set = set.add(&z3_e.element);
    }
    Ok(Z3Expression::new(&u.loc, &set))
}

/// _A bit hacky approach._
///
/// - If the access is for the state's member, then we lookup the constraint id in the its
///   local table.
/// - Otherwise, we just construct a variable name `{name}.{member_id}` and store in the
///   scope in case we refer to it later.
///
/// # Errors
/// - the member is access from non-variable expression.
fn member_access<'ctx>(
    e: &MemberAccess,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let Expression::Variable(var) = &e.expr.as_ref() else {
        params.diagnostics.push(Report::ver_error(
            e.expr.loc().clone(),
            String::from("Non-variable access is unsupported in verifier."),
        ));
        return Err(());
    };

    if let TypeVariant::State(s) = &var.ty {
        let local_scope = &mut params
            .executor
            .declarations
            .get_mut(&GlobalSymbol::State(s.clone()))
            .expect("Should exist")
            .scope;

        let state_decl = &params.contract.states[s.i];
        let members = state_decl.fields(params.contract);
        let member = &members[e.member.0];
        let c = local_scope
            .get(
                &member.name.name,
                type_to_sort(&member.ty.ty, params.ctx),
                params.ctx,
            )
            .expect("const should exist");

        return Ok(Z3Expression::new(&e.loc, &c));
    }

    let name = &params.scope.vars[var.element].ident.name;
    let variant = e.member.0.to_string();
    let c = params.z3_scope.create_or_get(
        &format!("{}.{}", name, variant),
        type_to_sort(&e.ty, params.ctx),
        params.ctx,
        params.executor,
    );

    Ok(Z3Expression::new(&e.loc, &c))
}

fn variable<'ctx>(
    e: &UnaryExpression<usize>,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let var = params.scope.vars.get(&e.element).expect("should exist");
    let z3_const = params.z3_scope.create_or_get(
        &var.ident.name,
        type_to_sort(&e.ty, params.ctx),
        params.ctx,
        params.executor,
    );
    Ok(Z3Expression::new(&e.loc, &z3_const))
}

pub fn type_to_sort<'ctx>(ty: &TypeVariant, ctx: &'ctx Context) -> Sort<'ctx> {
    match ty {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char | TypeVariant::Enum(_) => {
            Sort::int(ctx)
        }
        TypeVariant::Float => Sort::real(ctx),
        TypeVariant::Address | TypeVariant::Hex | TypeVariant::String => Sort::string(ctx),
        TypeVariant::Bool => Sort::bool(ctx),
        TypeVariant::Unit => Sort::uninterpreted(ctx, "()".into()),
        TypeVariant::Model(s) => Sort::uninterpreted(ctx, format!("M!{}", s.i).into()),
        TypeVariant::State(s) => Sort::uninterpreted(ctx, format!("S!{}", s.i).into()),
        TypeVariant::Struct(s) => Sort::uninterpreted(ctx, format!("SS!{}", s.i).into()),
        TypeVariant::Set(s_ty) => Sort::set(ctx, &type_to_sort(s_ty, ctx)),
        TypeVariant::List(l_ty) => Sort::set(ctx, &type_to_sort(l_ty, ctx)),
        TypeVariant::Mapping(m) => {
            Sort::array(
                ctx,
                &type_to_sort(&m.from_ty, ctx),
                &type_to_sort(&m.to_ty, ctx),
            )
        }
        TypeVariant::Function(_) => unimplemented!(),
        TypeVariant::Generic(_) => unimplemented!(),
    }
}

fn int_real_op<'ctx>(
    e: &Expression,
    params: &mut TransformParams<'ctx, '_>,
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
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;
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
                Expression::Less(_) => Dynamic::from_ast(&n1.lt(&n2)),
                Expression::LessEq(_) => Dynamic::from_ast(&n1.le(&n2)),
                Expression::Greater(_) => Dynamic::from_ast(&n1.gt(&n2)),
                Expression::GreaterEq(_) => Dynamic::from_ast(&n1.ge(&n2)),
                _ => unreachable!(),
            }
        }
        (_, _, Ok(n1), Ok(n2)) => {
            match e {
                Expression::Add(_) => Dynamic::from_ast(&(n1 + n2)),
                Expression::Subtract(_) => Dynamic::from_ast(&(n1 - n2)),
                Expression::Multiply(_) => Dynamic::from_ast(&(n1 * n2)),
                Expression::Divide(_) => Dynamic::from_ast(&(n1 / n2)),
                Expression::Less(_) => Dynamic::from_ast(&n1.lt(&n2)),
                Expression::LessEq(_) => Dynamic::from_ast(&n1.le(&n2)),
                Expression::Greater(_) => Dynamic::from_ast(&n1.gt(&n2)),
                Expression::GreaterEq(_) => Dynamic::from_ast(&n1.ge(&n2)),
                _ => unreachable!(),
            }
        }
        _ => {
            params.diagnostics.push(Report::ver_error_with_extra(
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
    b: &BinaryExpression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;

    let mut reports = Vec::new();
    let int1 = to_z3_int(&e1, &mut reports);
    let int2 = to_z3_int(&e2, &mut reports);
    match (int1, int2) {
        (Ok(n1), Ok(n2)) => {
            let res = n1 % n2;
            Ok(Z3Expression::new(&b.loc, &res))
        }
        _ => {
            params.diagnostics.push(Report::ver_error_with_extra(
                b.loc.clone(),
                String::from("Can not perform modulo operation."),
                reports,
            ));
            Err(())
        }
    }
}

fn equality<'ctx>(
    b: &BinaryExpression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;

    let res = e1.element._safe_eq(&e2.element).map_err(|_| {
        params.diagnostics.push(Report::ver_error(
            b.loc.clone(),
            String::from("Sort mismatch."),
        ))
    })?;

    Ok(Z3Expression::new(&b.loc, &res))
}

fn inequality<'ctx>(
    b: &BinaryExpression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;

    let res = Dynamic::distinct(params.ctx, &[&e1.element, &e2.element]);

    Ok(Z3Expression::new(&b.loc, &res))
}

fn not<'ctx>(
    u: &UnaryExpression<Box<Expression>>,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let v = transform_expr(&u.element, params)?;

    let bool_v = to_z3_bool(&v, params.diagnostics)?;

    Ok(Z3Expression::new(&u.loc, &bool_v))
}

fn or<'ctx>(
    b: &BinaryExpression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;

    let mut reports = Vec::new();
    let int1 = to_z3_bool(&e1, &mut reports);
    let int2 = to_z3_bool(&e2, &mut reports);
    match (int1, int2) {
        (Ok(n1), Ok(n2)) => {
            let res = Bool::or(params.ctx, &[n1, n2]);
            Ok(Z3Expression::new(&b.loc, &res))
        }
        _ => {
            params.diagnostics.push(Report::ver_error_with_extra(
                b.loc.clone(),
                String::from("Can not perform boolean OR."),
                reports,
            ));
            Err(())
        }
    }
}

fn and<'ctx>(
    b: &BinaryExpression,
    params: &mut TransformParams<'ctx, '_>,
) -> Result<Z3Expression<'ctx>, ()> {
    let e1 = transform_expr(&b.left, params)?;
    let e2 = transform_expr(&b.right, params)?;

    let mut reports = Vec::new();
    let int1 = to_z3_bool(&e1, &mut reports);
    let int2 = to_z3_bool(&e2, &mut reports);
    match (int1, int2) {
        (Ok(n1), Ok(n2)) => {
            let res = Bool::and(params.ctx, &[n1, n2]);
            Ok(Z3Expression::new(&b.loc, &res))
        }
        _ => {
            params.diagnostics.push(Report::ver_error_with_extra(
                b.loc.clone(),
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

/// Similar approach to 'member_access()', instead we use concrete variant name.
fn enum_<'ctx>(
    e: &UnaryExpression<usize>,
    params: &mut TransformParams<'ctx, '_>,
) -> Z3Expression<'ctx> {
    let TypeVariant::Enum(s) = &e.ty else {
        unreachable!("type must be enum");
    };
    let enum_ = &params.contract.enums[s.i];
    let name = &enum_.name.name;
    let variant = enum_
        .variants
        .get_index(e.element)
        .expect("variant should exist")
        .0;
    let c = params.z3_scope.create_or_get(
        &format!("{}.{}", name, variant),
        Sort::int(params.ctx),
        params.ctx,
        params.executor,
    );
    Z3Expression::new(&e.loc, &c)
}

fn to_z3_int<'ctx>(
    expr: &Z3Expression<'ctx>,
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
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
    executor: &mut SymbolicExecutor<'ctx>,
) -> (Bool<'ctx>, u32) {
    let val = executor.create_constant(&Sort::bool(ctx));
    (val.0.as_bool().unwrap(), val.1)
}
