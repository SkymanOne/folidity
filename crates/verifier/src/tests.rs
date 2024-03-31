use folidity_semantics::{
    ast::{
        BinaryExpression,
        Expression,
        TypeVariant,
        UnaryExpression,
    },
    Span,
};
use num_bigint::{
    BigInt,
    BigUint,
};
use num_traits::FromPrimitive;
use z3::{
    ast::{
        Ast,
        Int,
        Set,
        String as Z3String,
    },
    Context,
    SatResult,
    Solver,
    Sort,
};

use crate::{
    executor::SymbolicExecutor,
    transformer::transform_expr,
    z3_cfg,
};

#[test]
fn mul_transform() {
    let loc = Span { start: 0, end: 0 };
    let e1 = Expression::UInt(UnaryExpression {
        loc: loc.clone(),
        element: BigUint::from_i64(100).unwrap(),
        ty: TypeVariant::Int,
    });
    let e2 = Expression::UInt(UnaryExpression {
        loc: loc.clone(),
        element: BigUint::from_i64(2).unwrap(),
        ty: TypeVariant::Int,
    });
    let mul = Expression::Multiply(BinaryExpression {
        loc: loc.clone(),
        left: Box::new(e1),
        right: Box::new(e2),
        ty: TypeVariant::Int,
    });

    let context = Context::new(&z3_cfg());
    let mut executor = SymbolicExecutor::new(Context::new(&z3_cfg()));
    let mut diagnostics = vec![];
    let z3_res = transform_expr(&mul, &context, &mut diagnostics, &mut executor);

    assert!(z3_res.is_ok());
    let z3_e = z3_res.expect("Should be Ok");
    assert_eq!(
        z3_e.element.as_int(),
        Some(Int::from_i64(&context, 100) * Int::from_i64(&context, 2))
    );
}

#[test]
fn var_transform() {
    let loc = Span { start: 0, end: 0 };
    let var = Expression::Variable(UnaryExpression {
        loc: loc.clone(),
        element: 0,
        ty: TypeVariant::Int,
    });

    let context = Context::new(&z3_cfg());
    let mut executor = SymbolicExecutor::new(Context::new(&z3_cfg()));
    let mut diagnostics = vec![];
    let z3_res = transform_expr(&var, &context, &mut diagnostics, &mut executor);

    assert!(z3_res.is_ok());
    let z3_e = z3_res.expect("Should be Ok");
    assert_eq!(z3_e.element.as_int(), Some(Int::new_const(&context, 0)));
}

#[test]
fn string_hex_transform() {
    let loc = Span { start: 0, end: 0 };
    let string = Expression::String(UnaryExpression {
        loc: loc.clone(),
        element: "Hello World".to_string(),
        ty: TypeVariant::Int,
    });

    let context = Context::new(&z3_cfg());
    let mut executor = SymbolicExecutor::new(Context::new(&z3_cfg()));
    let mut diagnostics = vec![];
    let z3_res = transform_expr(&string, &context, &mut diagnostics, &mut executor);

    assert!(z3_res.is_ok());
    let z3_e = z3_res.expect("Should be Ok");
    assert_eq!(
        z3_e.element.as_string(),
        Some(Z3String::from_str(&context, "Hello World").unwrap())
    );

    let hex = Expression::Hex(UnaryExpression {
        loc: loc.clone(),
        element: hex::decode("AB").unwrap(),
        ty: TypeVariant::Int,
    });
    let z3_res = transform_expr(&hex, &context, &mut diagnostics, &mut executor);

    assert!(z3_res.is_ok());
    let z3_e = z3_res.expect("Should be Ok");
    assert_eq!(
        z3_e.element.as_string(),
        Some(Z3String::from_str(&context, "ab").unwrap())
    );
}

#[test]
fn list_transform() {
    let loc = Span { start: 0, end: 0 };
    let list = Expression::List(UnaryExpression {
        loc: loc.clone(),
        element: vec![
            Expression::Int(UnaryExpression {
                loc: loc.clone(),
                element: BigInt::from_i32(10).unwrap(),
                ty: TypeVariant::Int,
            }),
            Expression::Int(UnaryExpression {
                loc: loc.clone(),
                element: BigInt::from_i32(20).unwrap(),
                ty: TypeVariant::Int,
            }),
            Expression::Int(UnaryExpression {
                loc: loc.clone(),
                element: BigInt::from_i32(30).unwrap(),
                ty: TypeVariant::Int,
            }),
        ],
        ty: TypeVariant::Int,
    });

    let context = Context::new(&z3_cfg());
    let mut executor = SymbolicExecutor::new(Context::new(&z3_cfg()));
    let mut diagnostics = vec![];
    let z3_res = transform_expr(&list, &context, &mut diagnostics, &mut executor);

    assert!(z3_res.is_ok());
    let z3_e = z3_res.expect("Should be Ok");
    let mut set = Set::empty(&context, &Sort::int(&context));
    let elems = [
        Int::from_i64(&context, 10),
        Int::from_i64(&context, 20),
        Int::from_i64(&context, 30),
    ];
    for e in &elems {
        set = set.add(e);
    }
    assert_eq!(z3_e.element.as_set(), Some(set.clone()));

    let parsed_set = z3_e.element.as_set().expect("must be set");

    let solver = Solver::new(&context);
    solver.push();
    solver.assert(&parsed_set._eq(&set));
    assert_eq!(solver.check(), SatResult::Sat);

    solver.pop(1);
    solver.push();

    let distinct_set = set.del(&Int::from_i64(&context, 10));
    solver.assert(&parsed_set._eq(&distinct_set).not());
    assert_eq!(
        solver.check(),
        SatResult::Sat,
        "{:#?}",
        solver.get_assertions()
    );
}

#[test]
fn in_transform() {
    let loc = Span { start: 0, end: 0 };
    let list = Expression::List(UnaryExpression {
        loc: loc.clone(),
        element: vec![
            Expression::Int(UnaryExpression {
                loc: loc.clone(),
                element: BigInt::from_i32(10).unwrap(),
                ty: TypeVariant::Int,
            }),
            Expression::Int(UnaryExpression {
                loc: loc.clone(),
                element: BigInt::from_i32(20).unwrap(),
                ty: TypeVariant::Int,
            }),
            Expression::Int(UnaryExpression {
                loc: loc.clone(),
                element: BigInt::from_i32(30).unwrap(),
                ty: TypeVariant::Int,
            }),
        ],
        ty: TypeVariant::Int,
    });

    let member_in = Expression::Int(UnaryExpression {
        loc: loc.clone(),
        element: BigInt::from_i32(30).unwrap(),
        ty: TypeVariant::Int,
    });

    let member_out = Expression::Int(UnaryExpression {
        loc: loc.clone(),
        element: BigInt::from_i32(40).unwrap(),
        ty: TypeVariant::Int,
    });

    let in_true = Expression::In(BinaryExpression {
        loc: loc.clone(),
        left: Box::new(member_in),
        right: Box::new(list.clone()),
        ty: TypeVariant::Bool,
    });

    let in_false = Expression::In(BinaryExpression {
        loc: loc.clone(),
        left: Box::new(member_out),
        right: Box::new(list),
        ty: TypeVariant::Bool,
    });

    let context = Context::new(&z3_cfg());
    let mut executor = SymbolicExecutor::new(Context::new(&z3_cfg()));
    let mut diagnostics = vec![];

    // verify that `30` is in the list.
    let solver = Solver::new(&context);
    let z3_res = transform_expr(&in_true, &context, &mut diagnostics, &mut executor);
    assert!(z3_res.is_ok());
    let z3_in_true = z3_res.expect("Ok");
    solver.push();
    solver.assert(&z3_in_true.element.as_bool().expect("Should be bool."));
    assert_eq!(solver.check(), SatResult::Sat);

    solver.pop(1);
    solver.push();

    // verify that `40` is not in the list.
    let z3_res = transform_expr(&in_false, &context, &mut diagnostics, &mut executor);
    assert!(z3_res.is_ok());
    let z3_in_true = z3_res.expect("Ok");
    solver.assert(&z3_in_true.element.as_bool().expect("Should be bool.").not());
    assert_eq!(solver.check(), SatResult::Sat);
}
