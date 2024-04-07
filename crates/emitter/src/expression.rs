use algonaut_core::Address;
use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::{
    ast::{
        BinaryExpression,
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
    scratch: &'a mut ScratchTable,
    diagnostics: &'a mut Vec<Report>,
    emitter: &'a mut TealEmitter<'a>,
}

/// Emit expression
pub fn emit_expression(
    expr: &Expression,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    match expr {
        Expression::Variable(u) => var(u, chunks, args),

        // literals
        Expression::Int(u) => int(&u.element, &u.loc, chunks, args),
        Expression::UInt(u) => {
            int(
                &u.element.to_bigint().expect("always `Some`"),
                &u.loc,
                chunks,
                args,
            )
        }
        Expression::Boolean(u) => bool(u, chunks, args),
        Expression::Char(u) => char(u, chunks, args),
        Expression::String(u) => string(u, chunks, args),
        Expression::Hex(u) => hex(u, chunks, args),
        Expression::Address(u) => address(u, chunks, args),
        Expression::Enum(u) => enum_(u, chunks, args),
        Expression::Float(u) => float(u, chunks, args),

        // operations
        Expression::Add(b) => add(b, chunks, args),
        Expression::Subtract(b) => sub(b, chunks, args),
        Expression::Multiply(b) => mul(b, chunks, args),
        Expression::Divide(b) => div(b, chunks, args),
        Expression::Modulo(b) => modulo(b, chunks, args),
        Expression::Equal(b) => eq(b, chunks, args),
        Expression::NotEqual(b) => neq(b, chunks, args),
        Expression::Greater(b) => ge(b, chunks, args),
        Expression::Less(b) => le(b, chunks, args),
        Expression::GreaterEq(b) => geq(b, chunks, args),
        Expression::LessEq(b) => leq(b, chunks, args),
        Expression::Not(u) => not(u, chunks, args),
        Expression::Or(b) => or(b, chunks, args),
        Expression::And(b) => and(b, chunks, args),

        // Complex
        Expression::In(b) => todo!(),
        Expression::FunctionCall(_) => todo!(),
        Expression::MemberAccess(_) => todo!(),
        Expression::StructInit(_) => todo!(),
        Expression::List(_) => todo!(),
    }
}

fn add(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    // `left + right` should appear in stack as: `left => right => +`

    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint => Instruction::BPlus,
        TypeVariant::String => Instruction::Concat,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn sub(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    // `left - right` should appear in stack as: `left => right => -`

    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint => Instruction::BMinus,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn mul(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    // `left * right` should appear in stack as: `left => right => *`

    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint => Instruction::BMul,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn div(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    // `left / right` should appear in stack as: `left => right => /`

    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint => Instruction::BDiv,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn modulo(
    b: &BinaryExpression,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint => Instruction::BMod,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn le(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char => Instruction::BLess,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn leq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char => Instruction::BLessEq,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn ge(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char => Instruction::BMore,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn geq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char => Instruction::BMoreEq,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not yet supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn eq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    // `left == right` should appear in stack as: `left => right => ==`
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    local_chunks.push(Chunk::new_empty(Instruction::BEq));
    chunks.extend(local_chunks);

    Ok(())
}

fn neq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    // `left != right` should appear in stack as: `left => right => !=`
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    local_chunks.push(Chunk::new_empty(Instruction::BNeq));
    chunks.extend(local_chunks);

    Ok(())
}

fn not(
    u: &UnaryExpression<Box<Expression>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&u.element, &mut local_chunks, args)?;

    let op = match &u.ty {
        TypeVariant::Bool => Instruction::Not,
        _ => {
            args.diagnostics.push(Report::emit_error(
                u.loc.clone(),
                "This type is not supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn or(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Bool => Instruction::Or,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

fn and(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.ty {
        TypeVariant::Bool => Instruction::And,
        _ => {
            args.diagnostics.push(Report::emit_error(
                b.loc.clone(),
                "This type is not supported".to_string(),
            ));
            return Err(());
        }
    };

    local_chunks.push(Chunk::new_empty(op));
    chunks.extend(local_chunks);

    Ok(())
}

/// Load var from the scratch.
fn var(
    u: &UnaryExpression<usize>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let Some(var) = args.scratch.get_var(u.element) else {
        args.diagnostics.push(Report::emit_error(
            u.loc.clone(),
            String::from("Variable does not exist."),
        ));
        return Err(());
    };

    let c = Constant::Uint(var.index as u64);
    let chunk = Chunk::new_single(Instruction::Load, c);

    chunks.push(chunk);

    Ok(())
}

/// Handle signed and unsigned integers as bytes.
fn int(n: &BigInt, loc: &Span, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<(), ()> {
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
    chunks.push(chunk);

    Ok(())
}

/// Handle boolean values as `1` and `0` in Teal.
fn bool(
    u: &UnaryExpression<bool>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let val: u64 = if u.element { 1 } else { 0 };
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(())
}

/// Handle character as u64 value.
fn char(
    u: &UnaryExpression<char>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let val: u64 = u.element.into();
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(())
}

/// Handle raw string literals.
fn string(
    u: &UnaryExpression<String>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let c = Constant::String(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(())
}

/// Handle hex value bytes.
fn hex(
    u: &UnaryExpression<Vec<u8>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let c = Constant::Bytes(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(())
}

/// Handle Algorand address.
fn address(
    u: &UnaryExpression<Address>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let c = Constant::Addr(u.element.to_string());
    let chunk = Chunk::new_single(Instruction::PushAddr, c);
    chunks.push(chunk);

    Ok(())
}

/// Handle enum literals, we construct it from `bytes(enums_name + byte(variant_number)`
fn enum_(
    u: &UnaryExpression<usize>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
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
    chunks.push(chunk);

    Ok(())
}

/// Handle rational literals, for now we simply present them at f64 IEEE 754 standard.
fn float(
    u: &UnaryExpression<BigRational>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
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
    chunks.push(chunk);

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
