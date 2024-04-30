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
use num_bigint::{
    BigInt,
    BigUint,
};
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
        ty: TypeVariant::Uint,
    });
    let e2 = Expression::UInt(UnaryExpression {
        loc: loc.clone(),
        element: BigUint::from_i64(2).unwrap(),
        ty: TypeVariant::Uint,
    });
    let mul = Expression::Multiply(BinaryExpression {
        loc: loc.clone(),
        left: Box::new(e1),
        right: Box::new(e2),
        ty: TypeVariant::Uint,
    });

    let mut chunks = vec![];
    let res = emit_expression(&mul, &mut chunks, &mut args);
    assert!(res.is_ok());

    let expected = vec![
        Chunk {
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(100)],
        },
        Chunk {
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(2)],
        },
        Chunk {
            op: Instruction::Mul,
            constants: vec![],
        },
    ];

    assert_eq!(chunks, expected)
}

#[test]
fn signed_mul() {
    let definition = ContractDefinition::default();
    let mut emitter = TealEmitter::new(&definition);
    let loc = Span { start: 0, end: 0 };

    let mut args = EmitArgs {
        scratch: &mut ScratchTable::default(),
        diagnostics: &mut vec![],
        emitter: &mut emitter,
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

    let e1 = Expression::Int(UnaryExpression {
        loc: loc.clone(),
        element: BigInt::from_i64(100).unwrap(),
        ty: TypeVariant::Int,
    });
    let e2 = Expression::Int(UnaryExpression {
        loc: loc.clone(),
        element: BigInt::from_i64(-2).unwrap(),
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
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(16)],
        },
        Chunk {
            op: Instruction::ArrayInit,
            constants: vec![],
        },
        Chunk {
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(100)],
        },
        Chunk {
            op: Instruction::Replace,
            constants: vec![Constant::Uint(8)],
        },
        Chunk {
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(16)],
        },
        Chunk {
            op: Instruction::ArrayInit,
            constants: vec![],
        },
        Chunk {
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(2)],
        },
        Chunk {
            op: Instruction::Replace,
            constants: vec![Constant::Uint(8)],
        },
        Chunk {
            op: Instruction::PushInt,
            constants: vec![Constant::Uint(1)],
        },
        Chunk {
            op: Instruction::Replace,
            constants: vec![Constant::Uint(0)],
        },
        Chunk {
            op: Instruction::CallSub,
            constants: vec![Constant::StringLit("signed_mul".to_string())],
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

const COMPLEX_SRC: &str = r#"
state CounterState {
    counter: int,
} st [
    # example bounds
    counter < 1000,
    counter > -1000
]

# This is an constructor.
@init
# Anyone can call this function.
@(any)
fn () initialise() when () -> CounterState {
    loops(5.0);
    conditionals(false, 10);
    move_state();
    move CounterState : { 0 };
}

@(any)
fn () incr_by(value: int) when (CounterState s) -> CounterState
st [
    value > 100,
] {
    let value = s.counter + value;
    move CounterState : { value };
}

@(any)
fn () decr_by(value: int) when (CounterState s) -> CounterState 
st [
    value > 100,
] {
    let value = s.counter - value;
    move CounterState : { value };
}

@(any)
view(CounterState s) fn int get_value() {
    return s.counter;
}

fn () loops(value: float) {
    for (let mut i = 0; i < 10; i + 1) {
        let value = value + 123.0;
        skip;
    }
    let some_list = [-3, 4, 5];
}

fn () conditionals(cond: bool, value: int) {
    let scoped = -10;
    let mut s = s"Hello";
    s = s + s" " + s"World";
    if cond {
        let a = scoped + 3;
    } else if value > 1 {
        let b = scoped + 4;
    } else {
        let c = scoped + 5;
    }
}

fn () move_state() when (CounterState s1) -> (CounterState s2) {
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let b = [1, 2, 3];
    let c = -5;
    let d = s"Hello World";

    let counter = s1.counter;


    move CounterState : { counter };
}
"#;

#[test]
fn test_complex_emit() {
    folidity_diagnostics::disable_pretty_print();
    let result = folidity_parser::parse(COMPLEX_SRC);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let res = ContractDefinition::run(tree);
    assert!(res.is_ok(), "{:#?}", res.err().unwrap());
    let contract = res.unwrap();

    let runner = TealEmitter::run(&contract);

    assert!(runner.is_ok(), "{:#?}", runner.err().unwrap());
}
