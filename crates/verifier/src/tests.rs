use folidity_parser::ast::Identifier;
use folidity_semantics::{
    ast::{
        BinaryExpression,
        Expression,
        TypeVariant,
        UnaryExpression,
    },
    symtable::{
        Scope,
        VariableKind,
    },
    CompilationError,
    ContractDefinition,
    Runner,
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
    ast::Z3Scope,
    executor::SymbolicExecutor,
    transformer::{
        transform_expr,
        TransformParams,
    },
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
    let mut executor = SymbolicExecutor::new(&context);
    let mut diagnostics = vec![];
    let scope = Scope::default();
    let mut z3_scope = Z3Scope::default();
    let contract = ContractDefinition::default();
    let mut params = TransformParams {
        ctx: &context,
        z3_scope: &mut z3_scope,
        scope: &scope,
        contract: &contract,
        diagnostics: &mut diagnostics,
        executor: &mut executor,
    };
    let z3_res = transform_expr(&mul, &mut params);

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
    let mut executor = SymbolicExecutor::new(&context);
    let mut diagnostics = vec![];
    let mut scope = Scope::default();
    let mut contract = ContractDefinition::default();
    scope.add(
        &Identifier::new(0, 0, "a".to_string()),
        TypeVariant::Int,
        None,
        VariableKind::Param,
        false,
        0,
        &mut contract,
    );
    let mut z3_scope = Z3Scope::default();
    let mut params = TransformParams {
        ctx: &context,
        z3_scope: &mut z3_scope,
        scope: &scope,
        contract: &contract,
        diagnostics: &mut diagnostics,
        executor: &mut executor,
    };
    let z3_res = transform_expr(&var, &mut params);

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
    let mut executor = SymbolicExecutor::new(&context);
    let mut diagnostics = vec![];
    let scope = Scope::default();
    let mut z3_scope = Z3Scope::default();
    let contract = ContractDefinition::default();
    let mut params = TransformParams {
        ctx: &context,
        z3_scope: &mut z3_scope,
        scope: &scope,
        contract: &contract,
        diagnostics: &mut diagnostics,
        executor: &mut executor,
    };
    let z3_res = transform_expr(&string, &mut params);

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
    let z3_res = transform_expr(&hex, &mut params);

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
    let mut executor = SymbolicExecutor::new(&context);
    let mut diagnostics = vec![];
    let scope = Scope::default();
    let mut z3_scope = Z3Scope::default();
    let contract = ContractDefinition::default();
    let mut params = TransformParams {
        ctx: &context,
        z3_scope: &mut z3_scope,
        scope: &scope,
        contract: &contract,
        diagnostics: &mut diagnostics,
        executor: &mut executor,
    };
    let z3_res = transform_expr(&list, &mut params);

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
    let mut executor = SymbolicExecutor::new(&context);
    let mut diagnostics = vec![];
    let scope = Scope::default();
    let mut z3_scope = Z3Scope::default();
    let contract = ContractDefinition::default();
    let mut params = TransformParams {
        ctx: &context,
        z3_scope: &mut z3_scope,
        scope: &scope,
        contract: &contract,
        diagnostics: &mut diagnostics,
        executor: &mut executor,
    };

    // verify that `30` is in the list.
    let solver = Solver::new(&context);
    let z3_res = transform_expr(&in_true, &mut params);
    assert!(z3_res.is_ok());
    let z3_in_true = z3_res.expect("Ok");
    solver.push();
    solver.assert(&z3_in_true.element.as_bool().expect("Should be bool."));
    assert_eq!(solver.check(), SatResult::Sat);

    solver.pop(1);
    solver.push();

    // verify that `40` is not in the list.
    let z3_res = transform_expr(&in_false, &mut params);
    assert!(z3_res.is_ok());
    let z3_in_true = z3_res.expect("Ok");
    solver.assert(&z3_in_true.element.as_bool().expect("Should be bool.").not());
    assert_eq!(solver.check(), SatResult::Sat);
}

const WORKING: &str = r#"

model ParentModel {
    a: address,
    b: list<int>    
} st [
    a == a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY",
]

model MyModel: ParentModel {
    c: int,
    d: string
} st [
    a == a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY",
    c > -10,
    d == s"Hello World"
]

state StartState(MyModel) st [
    c < 1000
]

@init
@(any)
fn (r: bool) start(init: int) when () -> (StartState s) 
st [
    r == true,
    s.c < 10,
]
{
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let h = hex"1234";
    let b = [1, 2, 3];
    let c = -5;
    let d = s"Hello World";

    move StartState : { a, b, c, d };
    return true;
}

@(any)
view(StartState s) fn int get_value() {
    return s.c;
}
"#;

#[test]
fn test_correct_bounds() {
    folidity_diagnostics::disable_pretty_print();
    let result = folidity_parser::parse(WORKING);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let res = ContractDefinition::run(tree);
    assert!(res.is_ok(), "{:#?}", res.err().unwrap());
    let contract = res.unwrap();

    let runner = SymbolicExecutor::run(&contract);

    assert!(runner.is_ok(), "{:#?}", runner.err().unwrap());
}

const NOT_WORKING: &str = r#"

model ParentModel {
    a: address,
    b: list<int>,
    c: int
} st [
    # this works because `MyModel` refines the constraint block.
    c < 5
]

model MyModel: ParentModel {
    d: int,
    e: string
} st [
    a == a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY",
    c > 10,
    d > c,
    d < 5,
    e == s"Hello World"
]

state StartState(MyModel) st [
    c < 1000
]

@init
@(any)
fn (r: bool) start(init: int) when () -> (StartState s) 
st [
    r == true,
    s.c < 10,
]
{
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let h = hex"1234";
    let b = [1, 2, 3];
    let c = -5;
    let d = 10;
    let e = s"Hello World";

    move StartState : { a, b, c, d, e };
    return true;
}
"#;

#[test]
fn test_incorrect_bounds() {
    folidity_diagnostics::disable_pretty_print();
    let result = folidity_parser::parse(NOT_WORKING);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let res = ContractDefinition::run(tree);
    assert!(res.is_ok(), "{:#?}", res.err().unwrap());
    let contract = res.unwrap();

    let runner = SymbolicExecutor::run(&contract);

    let Err(CompilationError::Formal(reports)) = runner else {
        panic!("Expected error");
    };

    let error = reports.first().expect("contain error");
    assert_eq!(
        &error.message,
        "model MyModel has unsatisfiable constraints."
    );
    assert_eq!(error.additional_info.len(), 3);
    let mut errs = error.additional_info.iter();
    let e = errs.next().unwrap();
    assert!(
        e.message
            .contains("This is a constraint 11. It contradicts [12, 13]"),
        "{}",
        e.message
    );
    let e = errs.next().unwrap();
    assert!(
        e.message
            .contains("This is a constraint 12. It contradicts [11, 13]"),
        "{}",
        e.message
    );
    let e = errs.next().unwrap();
    assert!(
        e.message
            .contains("This is a constraint 13. It contradicts [11, 12]"),
        "{}",
        e.message
    );
}

const NOT_WORKING_LINKED: &str = r#"
model ParentModel {
    x: int,
} st [
    x > -1000
]

state StartState(ParentModel) st [
    x < 1000
]

@init
@(any)
fn (r: bool) start(init: int) when () -> (StartState s) 
st [
    r == true,
    s.x > 1000,
]
{
    let m = ParentModel : { 10 };

    move StartState : { m };
    return true;
}
"#;

#[test]
fn test_incorrect_linked_bounds() {
    folidity_diagnostics::disable_pretty_print();
    let result = folidity_parser::parse(NOT_WORKING_LINKED);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let res = ContractDefinition::run(tree);
    assert!(res.is_ok(), "{:#?}", res.err().unwrap());
    let contract = res.unwrap();

    let runner = SymbolicExecutor::run(&contract);

    let Err(CompilationError::Formal(reports)) = runner else {
        panic!("Expected error");
    };

    let error = reports.first().expect("contain error");
    assert_eq!(
        &error.message,
        "Detected conflicting constraints in linked blocks. These are the linked blocks: state StartState, function start"
    );
    assert_eq!(error.additional_info.len(), 2);
    let mut errs = error.additional_info.iter();
    let e = errs.next().unwrap();
    assert!(
        e.message
            .contains("This is a constraint 3 in state StartState. It contradicts [6]"),
        "{}",
        e.message
    );
    let e = errs.next().unwrap();
    assert!(
        e.message
            .contains("This is a constraint 6 in function start. It contradicts [3]"),
        "{}",
        e.message
    );
}
