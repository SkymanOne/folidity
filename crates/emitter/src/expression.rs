use algonaut_core::Address;
use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::{
    ast::{
        Expression,
        TypeVariant,
        UnaryExpression,
    },
    symtable::Scope,
    ContractDefinition,
};
use num_bigint::{
    BigInt,
    Sign,
    ToBigInt,
};
use num_rational::BigRational;
use num_traits::ToPrimitive;

use crate::{
    instruction::{
        Chunk,
        Constant,
        Instruction,
    },
    scratch_table::ScratchTable,
    teal::TealEmitter,
};

/// Arguments for the expression emitter.
#[derive(Debug)]
pub struct EmitExprArgs<'a> {
    definition: &'a ContractDefinition,
    chunks: &'a mut Vec<Chunk>,
    scratch: &'a mut ScratchTable,
    diagnostics: &'a mut Vec<Report>,
    emitter: &'a mut TealEmitter<'a>,
}

/// Emit expression
pub fn emit_expression(expr: &Expression, args: &mut EmitExprArgs) -> Result<(), ()> {
    match expr {
        Expression::Variable(u) => var(u, args),

        // literals
        Expression::Int(u) => int(&u.element, &u.loc, args),
        Expression::UInt(u) => int(&u.element.to_bigint().expect("always `Some`"), &u.loc, args),
        Expression::Boolean(u) => bool(u, args),
        Expression::Char(u) => char(u, args),
        Expression::String(u) => string(u, args),
        Expression::Hex(u) => hex(u, args),
        Expression::Address(u) => address(u, args),
        Expression::Enum(u) => enum_(u, args),
        Expression::Float(u) => float(u, args),

        // operations
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
        Expression::In(_) => todo!(),
        Expression::Not(_) => todo!(),
        Expression::Or(_) => todo!(),
        Expression::And(_) => todo!(),

        // Complex
        Expression::FunctionCall(_) => todo!(),
        Expression::MemberAccess(_) => todo!(),
        Expression::StructInit(_) => todo!(),
        Expression::List(_) => todo!(),
    }
}

/// Load var from the scratch.
fn var(u: &UnaryExpression<usize>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let Some(var) = args.scratch.get_var(u.element) else {
        args.diagnostics.push(Report::emit_error(
            u.loc.clone(),
            String::from("Variable does not exist."),
        ));
        return Err(());
    };

    let c = Constant::Uint(var.index as u64);
    let chunk = Chunk::new_single(Instruction::Load, c);

    args.chunks.push(chunk);

    Ok(())
}

/// Handle signed and unsigned integers as bytes.
fn int(n: &BigInt, loc: &Span, args: &mut EmitExprArgs) -> Result<(), ()> {
    let (sign, mut bytes) = n.to_bytes_be();
    if matches!(sign, Sign::Minus) {
        bytes = bytes.iter().map(|b| !b).collect();
        if !add_one_to_integer(&mut bytes) {
            args.diagnostics.push(Report::emit_error(
                loc.clone(),
                String::from("Overflow occurred"),
            ));
            return Err(());
        }
    }

    let c = Constant::Bytes(bytes);
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle boolean values as `1` and `0` in Teal.
fn bool(u: &UnaryExpression<bool>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let val: u64 = if u.element { 1 } else { 0 };
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle character as u64 value.
fn char(u: &UnaryExpression<char>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let val: u64 = u.element.into();
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle raw string literals.
fn string(u: &UnaryExpression<String>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let c = Constant::String(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle hex value bytes.
fn hex(u: &UnaryExpression<Vec<u8>>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let c = Constant::Bytes(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle Algorand address.
fn address(u: &UnaryExpression<Address>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let c = Constant::Addr(u.element.to_string());
    let chunk = Chunk::new_single(Instruction::PushAddr, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle enum literals, we construct it from `bytes(enums_name + byte(variant_number)`
fn enum_(u: &UnaryExpression<usize>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let TypeVariant::Enum(s) = &u.ty else {
        unreachable!()
    };

    let mut enum_name = args.definition.enums[s.i]
        .name
        .name
        .clone()
        .as_bytes()
        .to_vec();
    enum_name.push(u.element as u8);

    let c = Constant::Bytes(enum_name);
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Handle rational literals, for now we simply present them at f64 IEEE 754 standard.
fn float(u: &UnaryExpression<BigRational>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let Some(float_val) = u.element.to_f64() else {
        args.diagnostics.push(Report::emit_error(
            u.loc.clone(),
            String::from("Rational value is too large."),
        ));
        return Err(());
    };

    let bytes = float_val.to_bits().to_be_bytes();
    let c = Constant::Bytes(bytes.to_vec());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    args.chunks.push(chunk);

    Ok(())
}

/// Add `1` to bytes for Two's Complement Binary.
fn add_one_to_integer(bytes: &mut [u8]) -> bool {
    let mut carry = 1;

    for byte in bytes.iter_mut().rev() {
        if *byte == 0xFF && carry == 1 {
            *byte = 0x00;
        } else {
            *byte += carry;
            carry = 0;
            break;
        }
    }

    // If carry is still 1 here, it means the addition resulted in an overflow.
    carry == 1
}
