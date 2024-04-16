use folidity_semantics::{
    ast::{
        BinaryExpression,
        Expression,
        FuncReturnType,
        Function,
        FunctionVisibility,
        Type,
        TypeVariant,
        UnaryExpression,
    },
    ContractDefinition,
    Identifier,
    Runner,
    Span,
};
use indexmap::IndexMap;
use num_bigint::BigUint;
use num_traits::FromPrimitive;

use crate::{
    ast::{
        Chunk,
        Constant,
        Instruction,
    },
    expression::emit_expression,
    scratch_table::ScratchTable,
    teal::{
        EmitArgs,
        TealEmitter,
    },
};

#[test]
fn simple_exprs() {
    let definition = ContractDefinition::default();
    let mut emitter = TealEmitter::new(&definition);
    let loc = Span { start: 0, end: 0 };

    let mut args = EmitArgs {
        scratch: &mut ScratchTable::default(),
        diagnostics: &mut vec![],
        emitter: &mut emitter,
        concrete_vars: &mut IndexMap::default(),
        delayed_bounds: &mut vec![],
        func: &Function::new(
            loc.clone(),
            false,
            FunctionVisibility::Priv,
            FuncReturnType::Type(Type::default()),
            Identifier {
                loc: loc.clone(),
                name: "my_func".to_string(),
            },
            IndexMap::default(),
            None,
        ),
        loop_labels: &mut vec![],
    };

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

    let mut chunks = vec![];
    let res = emit_expression(&mul, &mut chunks, &mut args);
    assert!(res.is_ok());

    let expected = vec![
        Chunk {
            op: Instruction::PushBytes,
            constants: vec![Constant::Bytes(100_i128.to_be_bytes().to_vec())],
        },
        Chunk {
            op: Instruction::PushBytes,
            constants: vec![Constant::Bytes(2_i128.to_be_bytes().to_vec())],
        },
        Chunk {
            op: Instruction::BMul,
            constants: vec![],
        },
    ];

    assert_eq!(chunks, expected)
}

const WORKING_SIMPLE: &str = r#"

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
fn test_simple_emit() {
    folidity_diagnostics::disable_pretty_print();
    let result = folidity_parser::parse(WORKING_SIMPLE);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let res = ContractDefinition::run(tree);
    assert!(res.is_ok(), "{:#?}", res.err().unwrap());
    let contract = res.unwrap();

    let runner = TealEmitter::run(&contract);

    assert!(runner.is_ok(), "{:#?}", runner.err().unwrap());
}
