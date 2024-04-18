use algonaut_core::Address;
use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::{
    ast::{
        BinaryExpression,
        Bounds,
        Expression,
        FunctionCall,
        MemberAccess,
        Param,
        StateBody,
        StructInit,
        TypeVariant,
        UnaryExpression,
    },
    symtable::Scope,
};
use num_bigint::{
    BigInt,
    ToBigInt,
};
use num_rational::BigRational;
use num_traits::ToPrimitive;

use crate::{
    ast::{
        Chunk,
        Constant,
        Instruction,
        TypeSizeHint,
    },
    teal::EmitArgs,
};

type EmitResult = Result<u64, ()>;

/// Emit expression returning the len of the type in bytes.
pub fn emit_expression(
    expr: &Expression,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
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
        Expression::String(u) => string(u, chunks),
        Expression::Hex(u) => hex(u, chunks),
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
        Expression::FunctionCall(f) => func_call(f, chunks, args),
        Expression::In(b) => in_(b, chunks, args),
        Expression::MemberAccess(m) => member_access(m, chunks, args),
        Expression::StructInit(s) => struct_init(s, chunks, args),
        Expression::List(u) => list(u, chunks, args),
    }
}

// todo: write a support teal function to checking inclusion and use it here.
fn in_(b: &BinaryExpression, _chunks: &mut [Chunk], args: &mut EmitArgs) -> EmitResult {
    args.diagnostics.push(Report::emit_error(
        b.loc.clone(),
        "Unsupported currently".to_string(),
    ));
    Err(())
}

fn member_access(m: &MemberAccess, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    let _ = emit_expression(&m.expr, &mut local_chunks, args)?;

    let fields = match m.expr.ty() {
        TypeVariant::Struct(sym) => {
            let struct_decl = &args.emitter.definition.structs[sym.i];
            struct_decl.fields.clone()
        }
        TypeVariant::State(sym) => {
            let state_decl = &args.emitter.definition.states[sym.i];
            state_decl.fields(args.emitter.definition)
        }
        TypeVariant::Model(sym) => {
            let model_decl = &args.emitter.definition.models[sym.i];
            model_decl.fields(args.emitter.definition)
        }
        _ => {
            args.diagnostics.push(Report::emit_error(
                m.loc.clone(),
                "Other types are unsupported currently".to_string(),
            ));
            return Err(());
        }
    };

    extract_field(&fields, m.member.0, None, &mut local_chunks, args)?;

    chunks.extend(local_chunks);

    Ok(m.ty.size_hint(args.emitter.definition))
}

fn struct_init(s: &StructInit, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    match &s.ty {
        TypeVariant::Struct(sym) => {
            let struct_decl = &args.emitter.definition.structs[sym.i];
            init_array(
                s,
                &Scope::default(),
                &struct_decl.fields,
                &None,
                &mut local_chunks,
                args,
            )?;
        }
        TypeVariant::Model(sym) => {
            let model_decl = &args.emitter.definition.models[sym.i];
            init_array(
                s,
                &model_decl.scope,
                &model_decl.fields(args.emitter.definition),
                &model_decl.bounds,
                &mut local_chunks,
                args,
            )?;
        }
        TypeVariant::State(sym) => {
            let state_decl = &args.emitter.definition.states[sym.i];

            let Some(body) = &state_decl.body else {
                return Ok(0);
            };

            match body {
                StateBody::Raw(_) => {
                    init_array(
                        s,
                        &state_decl.scope,
                        &state_decl.fields(args.emitter.definition),
                        &state_decl.bounds,
                        &mut local_chunks,
                        args,
                    )?;
                }
                StateBody::Model(model_sym) => {
                    let model_decl = &args.emitter.definition.models[model_sym.i];
                    init_array(
                        s,
                        &model_decl.scope,
                        &model_decl.fields(args.emitter.definition),
                        &model_decl.bounds,
                        &mut local_chunks,
                        args,
                    )?;
                }
            }
        }
        _ => {
            args.diagnostics.push(Report::ver_error(
                s.loc.clone(),
                "This type is not supported for instantiation in emitter.".to_string(),
            ));
            return Err(());
        }
    }

    chunks.extend(local_chunks);
    Ok(s.ty.size_hint(args.emitter.definition))
}

fn init_array(
    s: &StructInit,
    scope: &Scope,
    fields: &[Param],
    bounds: &Option<Bounds>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> Result<(), ()> {
    let array_index = args.emitter.scratch_index_incr()?;
    let mut local_chunks = vec![];

    let array_size: u64 = s.ty.size_hint(args.emitter.definition);

    // create zero-filled array and store it
    local_chunks.extend_from_slice(&[
        Chunk::new_single(Instruction::PushInt, Constant::Uint(array_size)),
        Chunk::new_empty(Instruction::ArrayInit),
        Chunk::new_single(Instruction::Store, Constant::Uint(array_index)),
    ]);

    // iteratively parse each argument expression, and store it in the array.
    let mut loc_offset = (s.args.len() as u64 - 1) * 8;
    let mut data_offests = vec![];
    for a in &s.args {
        // push current offset to the list.
        data_offests.push(loc_offset);

        // emit expression
        let size = emit_expression(a, &mut local_chunks, args)?;

        // and store it temporarily.
        let data_index = args.emitter.scratch_index_incr()?;
        local_chunks.push(Chunk::new_single(
            Instruction::Store,
            Constant::Uint(data_index),
        ));

        if a.ty().is_resizable() {
            local_chunks.extend_from_slice(&[
                Chunk::new_single(Instruction::Load, Constant::Uint(array_index)), // load array
                Chunk::new_single(Instruction::PushInt, Constant::Uint(size)), // push actual size of data
                Chunk::new_single(Instruction::Replace, Constant::Uint(loc_offset)), // place it in the block
                Chunk::new_single(Instruction::Store, Constant::Uint(array_index)), // store the array
            ]);
            loc_offset += 8; // increment the offset by 8
        }
        local_chunks.extend_from_slice(&[
            Chunk::new_single(Instruction::Load, Constant::Uint(array_index)), // load array
            Chunk::new_single(Instruction::Load, Constant::Uint(data_index)),  // load data
            Chunk::new_single(Instruction::Replace, Constant::Uint(loc_offset)), /* place it in
                                                                                * the block */
            Chunk::new_single(Instruction::Store, Constant::Uint(array_index)), // store the array
        ]);

        // increment offset for the next block.
        loc_offset += a.ty().size_hint(args.emitter.definition);
    }

    // if there are bounds add them to the delay to be resolved after.
    if let Some(bounds) = bounds {
        // create concrete chunks and insert them in the lookup table.
        for (i, f) in fields.iter().enumerate() {
            let (p_no, _) = scope.find_var_index(&f.name.name).expect("should exist");
            let mut concrete_chunks = vec![];
            extract_field(fields, i, Some(array_index), &mut concrete_chunks, args)?;
            args.emitter.concrete_vars.insert(p_no, concrete_chunks);
        }

        let mut error = false;

        for e in &bounds.exprs {
            error |= emit_expression(e, &mut local_chunks, args).is_err();
            local_chunks.push(Chunk::new_empty(Instruction::Assert));
        }

        if error {
            args.diagnostics.push(Report::ver_error(
                bounds.loc.clone(),
                "Error emitting bounds during struct instantiation.".to_string(),
            ));
            return Err(());
        }
    }

    local_chunks.push(Chunk::new_single(
        Instruction::Load,
        Constant::Uint(array_index),
    ));
    chunks.extend(local_chunks);

    Ok(())
}

fn extract_field(
    fields: &[Param],
    member: usize,
    array_index: Option<u64>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
    let mut local_chunks = vec![];
    // save the content in the scratch.

    // extract array index if provided or create one if the struct wasn't stored before.
    let array_index = if let Some(index) = array_index {
        index
    } else {
        let index = args.emitter.scratch_index_incr()?;
        local_chunks.push(Chunk::new_single(Instruction::Store, Constant::Uint(index)));
        index
    };
    let mut offset_loc: u64 = 0;

    for (i, f) in fields.iter().enumerate() {
        if i == member {
            break;
        }
        offset_loc += f.ty.ty.size_hint(args.emitter.definition);
        if f.ty.ty.is_resizable() {
            offset_loc += 8; // add 8 to the offset to accommodate for the size block.
        }
    }

    local_chunks.extend_from_slice(&[
        Chunk::new_single(Instruction::Load, Constant::Uint(array_index)), /* load array from
                                                                            * memory */
        Chunk::new_single(Instruction::PushInt, Constant::Uint(offset_loc)), // push offset
    ]);

    let ty = &fields[member].ty.ty;
    if ty.is_resizable() {
        let size_index = args.emitter.scratch_index_incr()?;
        let data_loc = offset_loc + 8;
        local_chunks.extend_from_slice(&[
            Chunk::new_empty(Instruction::ExtractUint), // extract size data
            Chunk::new_single(Instruction::Store, Constant::Uint(size_index)), /* store size in
                                                         * scratch. */
            // handle accessing data
            Chunk::new_single(Instruction::Load, Constant::Uint(array_index)), /* load array
                                                                                * from memory */
            Chunk::new_single(Instruction::PushInt, Constant::Uint(data_loc)), /* push offset of
                                                                                * the actual
                                                                                * data */
            // Handle accessing size
            Chunk::new_single(Instruction::Load, Constant::Uint(size_index)), /* load array from
                                                                               * memory */
            //
            Chunk::new_empty(Instruction::Extract3), // extract data from array
        ]);
    } else if matches!(
        ty,
        TypeVariant::Int
            | TypeVariant::Uint
            | TypeVariant::Float
            | TypeVariant::Bool
            | TypeVariant::Char
    ) {
        local_chunks.push(Chunk::new_empty(Instruction::ExtractUint))
    } else {
        local_chunks.extend_from_slice(&[
            Chunk::new_single(
                Instruction::PushInt,
                Constant::Uint(ty.size_hint(args.emitter.definition)),
            ), // size
            Chunk::new_empty(Instruction::Extract3), // extract data
        ])
    }

    // args.emitter.scratch_index = array_index as u64; // reset index to preserve space.

    chunks.extend(local_chunks);

    Ok(0)
}

fn list(
    u: &UnaryExpression<Vec<Expression>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
    if u.element.is_empty() {
        chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(0)));
        chunks.push(Chunk::new_empty(Instruction::ArrayInit));
        return Ok(u.ty.size_hint(args.emitter.definition));
    }

    let mut list_chunks: Vec<Chunk> = vec![];
    let mut error = false;
    let mut size = 0;
    let first_elem = &u.element[0];
    if let Ok(s) = emit_expression(first_elem, &mut list_chunks, args) {
        // after every second element we want to concat them together
        size += s;
    } else {
        error |= true;
    }

    for e in u.element.iter().skip(1) {
        if let Ok(s) = emit_expression(e, &mut list_chunks, args) {
            // after first element we want to concat with the previous result.
            list_chunks.push(Chunk::new_empty(Instruction::Concat));
            size += s;
        } else {
            error |= true;
        }
    }

    if error {
        return Err(());
    }

    chunks.extend(list_chunks);

    Ok(size)
}

fn func_call(f: &FunctionCall, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut arg_chunks: Vec<Chunk> = vec![];

    let mut error = false;
    for e in &f.args {
        error |= emit_expression(e, &mut arg_chunks, args).is_err();
    }

    if error {
        return Err(());
    }

    chunks.extend(arg_chunks);

    let func_decl = &args.emitter.definition.functions[f.sym.i];

    // we use `__<name>` convention for function names.
    let name = format!("__{}", func_decl.name.name);
    chunks.push(Chunk::new_single(
        Instruction::CallSub,
        Constant::StringLit(name),
    ));

    Ok(f.returns.size_hint(args.emitter.definition))
}

fn add(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    // `left + right` should appear in stack as: `left => right => +`

    let mut local_chunks = vec![];
    let _ = emit_expression(&b.left, &mut local_chunks, args)?;
    let _ = emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Float => Instruction::Plus,
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn sub(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    // `left - right` should appear in stack as: `left => right => -`

    let mut local_chunks = vec![];
    let _ = emit_expression(&b.left, &mut local_chunks, args)?;
    let _ = emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Float => Instruction::Minus,
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn mul(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    // `left * right` should appear in stack as: `left => right => *`

    let mut local_chunks = vec![];
    let _ = emit_expression(&b.left, &mut local_chunks, args)?;
    let _ = emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Float => Instruction::Mul,
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn div(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    // `left / right` should appear in stack as: `left => right => /`

    let mut local_chunks = vec![];
    let _ = emit_expression(&b.left, &mut local_chunks, args)?;
    let _ = emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Float => Instruction::Div,
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn modulo(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    let _ = emit_expression(&b.left, &mut local_chunks, args)?;
    let _ = emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint => Instruction::Mod,
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn le(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char | TypeVariant::Float => {
            Instruction::Less
        }
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn leq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char | TypeVariant::Float => {
            Instruction::LessEq
        }
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn ge(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char | TypeVariant::Float => {
            Instruction::More
        }
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn geq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
        TypeVariant::Int | TypeVariant::Uint | TypeVariant::Char | TypeVariant::Float => {
            Instruction::MoreEq
        }
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn eq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    // `left == right` should appear in stack as: `left => right => ==`
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    local_chunks.push(Chunk::new_empty(Instruction::Eq));

    chunks.extend(local_chunks);

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn neq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    // `left != right` should appear in stack as: `left => right => !=`
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    local_chunks.push(Chunk::new_empty(Instruction::Neq));

    chunks.extend(local_chunks);

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn not(
    u: &UnaryExpression<Box<Expression>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
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

    Ok(u.ty.size_hint(args.emitter.definition))
}

fn or(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

fn and(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    let op = match &b.left.ty() {
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

    Ok(b.ty.size_hint(args.emitter.definition))
}

/// Load var from the scratch.
fn var(u: &UnaryExpression<usize>, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    if let Some(local_chunks) = args.emitter.concrete_vars.get(&u.element) {
        chunks.extend_from_slice(local_chunks);
        return Ok(0);
    }

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

    Ok(u.ty.size_hint(args.emitter.definition))
}

/// Handle signed and unsigned integers as bytes.
fn int(n: &BigInt, loc: &Span, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let Some(int_val) = n.to_i64() else {
        args.diagnostics.push(Report::emit_error(
            loc.clone(),
            String::from("Integer value is too large."),
        ));
        return Err(());
    };

    let bytes = int_val.to_be_bytes();
    let val = u64::from_be_bytes(bytes);
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(TypeVariant::Int.size_hint(args.emitter.definition))
}

/// Handle boolean values as `1` and `0` in Teal.
fn bool(u: &UnaryExpression<bool>, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let val: u64 = if u.element { 1 } else { 0 };
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(u.ty.size_hint(args.emitter.definition))
}

/// Handle character as u64 value.
fn char(u: &UnaryExpression<char>, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let val: u64 = u.element.into();
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(u.ty.size_hint(args.emitter.definition))
}

/// Handle raw string literals.
fn string(u: &UnaryExpression<String>, chunks: &mut Vec<Chunk>) -> EmitResult {
    let c = Constant::String(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(u.element.len() as u64)
}

/// Handle hex value bytes.
fn hex(u: &UnaryExpression<Vec<u8>>, chunks: &mut Vec<Chunk>) -> EmitResult {
    let c = Constant::Bytes(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(u.element.len() as u64)
}

/// Handle Algorand address.
fn address(
    u: &UnaryExpression<Address>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
    let c = Constant::StringLit(u.element.to_string());
    let chunk = Chunk::new_single(Instruction::PushAddr, c);
    chunks.push(chunk);

    Ok(u.ty.size_hint(args.emitter.definition))
}

/// Handle enum literals, we construct it from `bytes(enums_pos) ++ byte(variant_number)`
fn enum_(u: &UnaryExpression<usize>, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let TypeVariant::Enum(s) = &u.ty else {
        unreachable!()
    };

    let mut enum_value: Vec<u8> = vec![];
    enum_value.extend_from_slice(&s.i.to_be_bytes());
    enum_value.extend_from_slice(&u.element.to_be_bytes());

    let c = Constant::Bytes(enum_value);
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(u.ty.size_hint(args.emitter.definition))
}

/// Handle rational literals, for now we simply present them at f64 IEEE 754 standard.
fn float(
    u: &UnaryExpression<BigRational>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
    let Some(float_val) = u.element.to_f64() else {
        args.diagnostics.push(Report::emit_error(
            u.loc.clone(),
            String::from("Rational value is too large."),
        ));
        return Err(());
    };

    let bytes = float_val.to_bits().to_be_bytes();
    let val = u64::from_be_bytes(bytes);
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(u.ty.size_hint(args.emitter.definition))
}
