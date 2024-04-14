use folidity_semantics::{
    ast::{
        BinaryExpression,
        Expression,
        TypeVariant,
        UnaryExpression,
    },
    ContractDefinition,
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
    expression::{
        emit_expression,
        EmitExprArgs,
    },
    scratch_table::ScratchTable,
    teal::TealEmitter,
};

#[test]
fn simple_exprs() {
    let definition = ContractDefinition::default();
    let mut emitter = TealEmitter::new(&definition);

    let mut args = EmitExprArgs {
        scratch: &mut ScratchTable::default(),
        diagnostics: &mut vec![],
        emitter: &mut emitter,
        concrete_vars: &mut IndexMap::default(),
    };

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

    let mut chunks = vec![];
    let res = emit_expression(&mul, &mut chunks, &mut args);
    assert!(res.is_ok());

    let expected = vec![
        Chunk {
            op: Instruction::Empty,
            constants: vec![],
        },
        Chunk {
            op: Instruction::PushBytes,
            constants: vec![Constant::Bytes(vec![100])],
        },
        Chunk {
            op: Instruction::PushBytes,
            constants: vec![Constant::Bytes(vec![2])],
        },
        Chunk {
            op: Instruction::BMul,
            constants: vec![],
        },
        Chunk {
            op: Instruction::Empty,
            constants: vec![],
        },
    ];

    assert_eq!(chunks, expected)
}
