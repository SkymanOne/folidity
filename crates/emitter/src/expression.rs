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
use indexmap::IndexMap;
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
        TypeSizeHint,
    },
    scratch_table::ScratchTable,
    teal::TealEmitter,
};

/// Arguments for the expression emitter.
#[derive(Debug)]
pub struct EmitExprArgs<'a> {
    scratch: &'a mut ScratchTable,
    diagnostics: &'a mut Vec<Report>,
    emitter: &'a mut TealEmitter<'a>,
    concrete_vars: IndexMap<usize, Vec<Chunk>>,
}

/// Emit expression returning the len of the type in bytes.
pub fn emit_expression(
    expr: &Expression,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
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
        Expression::FunctionCall(f) => func_call(f, chunks, args),
        Expression::In(b) => todo!(),
        Expression::MemberAccess(m) => member_access(m, chunks, args),
        Expression::StructInit(s) => struct_init(s, chunks, args),
        Expression::List(u) => list(u, chunks, args),
    }
}

fn member_access(
    m: &MemberAccess,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let mut load_chunks = vec![];
    // load data arr
    let _ = emit_expression(&m.expr, &mut load_chunks, args)?;

    match m.expr.ty() {
        TypeVariant::Struct(sym) => {
            let struct_decl = &args.emitter.definition.structs[sym.i];
            extract_field_data(&struct_decl.fields, m.member.0, &load_chunks, chunks, args);
        }
        TypeVariant::Model(sym) => {
            let model_decl = &args.emitter.definition.models[sym.i];
            extract_field_data(
                &model_decl.fields(args.emitter.definition),
                m.member.0,
                &load_chunks,
                chunks,
                args,
            );
        }
        TypeVariant::State(sym) => {
            let state_decl = &args.emitter.definition.states[sym.i];
            extract_field_data(
                &state_decl.fields(args.emitter.definition),
                m.member.0,
                &load_chunks,
                chunks,
                args,
            );
        }
        _ => unimplemented!(),
    };

    Ok(0)
}

fn extract_field_data(
    fields: &[Param],
    f_n: usize,
    load_chunks: &[Chunk],
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) {
    let mut offset_chunks = vec![Chunk::new_single(Instruction::PushInt, Constant::Uint(0))];
    let current_i = args.emitter.scratch_index as u64;

    for (i, f) in fields.iter().enumerate() {
        if i == f_n {
            // load data
            offset_chunks.extend_from_slice(load_chunks);
            // load current offset from the temp location.
            offset_chunks.push(Chunk::new_single(
                Instruction::Load,
                Constant::Uint(current_i),
            ));
            match &f.ty.ty {
                // for bool and char, we can just extract u64 directly.
                TypeVariant::Bool | TypeVariant::Char => {
                    offset_chunks.push(Chunk::new_empty(Instruction::ExtractUint))
                }
                // for address we can extract 32 bytes precisely.
                TypeVariant::Address => {
                    offset_chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(32)));
                    offset_chunks.push(Chunk::new_empty(Instruction::Extract3));
                }
                // for any other types, we need to read size and then read the contents.
                _ => {
                    // Push 8 byte offset
                    offset_chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(8)));
                    // current_offset + 8 = offset of the size value for the field.
                    offset_chunks.push(Chunk::new_empty(Instruction::Plus));

                    // extract size value.
                    offset_chunks.push(Chunk::new_empty(Instruction::ExtractUint));

                    let size_i = current_i + 1;
                    // store the size value in the next temp location.
                    offset_chunks
                        .push(Chunk::new_single(Instruction::Load, Constant::Uint(size_i)));

                    // load data
                    offset_chunks.extend_from_slice(load_chunks);
                    // load current offset from the temp location.
                    offset_chunks.push(Chunk::new_single(
                        Instruction::Load,
                        Constant::Uint(current_i),
                    ));
                    // Push 16 byte offset
                    offset_chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(16)));
                    // current_offset + 16 = offset of the data of the field.
                    offset_chunks.push(Chunk::new_empty(Instruction::Plus));
                    // load the size value from scratch
                    offset_chunks
                        .push(Chunk::new_single(Instruction::Load, Constant::Uint(size_i)));
                    // extract data
                    offset_chunks.push(Chunk::new_empty(Instruction::Extract3));
                }
            };

            break;
        }

        if let Some(size) = f.ty.ty.size_hint() {
            offset_chunks.push(Chunk::new_single(
                Instruction::PushInt,
                Constant::Uint(size),
            ));
        } else {
            // use current index as a temporary location.
            // store current offset
            offset_chunks.push(Chunk::new_single(
                Instruction::Store,
                Constant::Uint(current_i),
            ));
            // insert load instruction for the storage.
            offset_chunks.extend_from_slice(load_chunks);
            // load current offset
            offset_chunks.push(Chunk::new_single(
                Instruction::Load,
                Constant::Uint(current_i),
            ));
            // duplicate offset
            offset_chunks.push(Chunk::new_empty(Instruction::Dup));
            // extract capacity uint64 capacity value for the field
            offset_chunks.push(Chunk::new_empty(Instruction::ExtractUint));
        }

        // add previous two values.
        offset_chunks.push(Chunk::new_empty(Instruction::Plus));
        // store value in temp location.
        offset_chunks.push(Chunk::new_single(
            Instruction::Store,
            Constant::Uint(current_i),
        ));
    }

    chunks.extend(offset_chunks);
}

fn struct_init(
    s: &StructInit,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    match &s.ty {
        TypeVariant::Struct(_) => {
            emit_init_args(s, chunks, args).map(|r| Ok(r.iter().sum::<u64>()))?
        }
        TypeVariant::Model(sym) => {
            let mut local_chunks = vec![];
            let model_decl = &args.emitter.definition.models[sym.i];
            struct_arr_load(
                s,
                &model_decl.scope,
                &model_decl.fields(args.emitter.definition),
                &model_decl.bounds,
                &mut local_chunks,
                args,
            )?;

            add_padding(&mut local_chunks);
            chunks.extend(local_chunks);

            Ok(0)
        }
        TypeVariant::State(sym) => {
            let state_decl = &args.emitter.definition.states[sym.i];
            let Some(body) = &state_decl.body else {
                // todo: init state.
                return Ok(0);
            };

            match body {
                StateBody::Raw(fields) => {
                    let mut local_chunks = vec![];
                    struct_arr_load(
                        s,
                        &state_decl.scope,
                        fields,
                        &state_decl.bounds,
                        &mut local_chunks,
                        args,
                    )?;
                }
                // todo: rethink approach
                StateBody::Model(sym_m) => {
                    let mut local_chunks = vec![];
                    let model_decl = &args.emitter.definition.models[sym_m.i];
                    struct_arr_load(
                        s,
                        &model_decl.scope,
                        &model_decl.fields(args.emitter.definition),
                        &model_decl.bounds,
                        &mut local_chunks,
                        args,
                    )?;

                    add_padding(&mut local_chunks);
                    chunks.extend(local_chunks);
                }
            };

            Ok(0)
        }
        _ => unreachable!(),
    }
}

fn struct_arr_load(
    s: &StructInit,
    scope: &Scope,
    fields: &[Param],
    bounds: &Option<Bounds>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    let sizes = emit_init_args(s, chunks, args)?;

    if let Some(bounds) = &bounds {
        let arr_index = args.emitter.scratch_index as u64;
        chunks.push(Chunk::new_single(
            Instruction::Store,
            Constant::Uint(arr_index),
        ));

        for (i, f) in fields.iter().enumerate() {
            let (var_n, _) = &scope.find_var_index(&f.name.name).expect("should exist");

            let load_chunk = Chunk::new_single(Instruction::Load, Constant::Uint(arr_index));

            if f.ty.ty.is_resizable() {
                let mut skip_size = sizes.iter().take(i).sum::<u64>() + 8;
                // get size int
                let skip_size_chunk =
                    Chunk::new_single(Instruction::PushInt, Constant::Uint(skip_size));
                let extract_int_chunk = Chunk::new_empty(Instruction::ExtractUint);
                skip_size += 8;
                let skip_data_chunk =
                    Chunk::new_single(Instruction::PushInt, Constant::Uint(skip_size));

                // load arr -> push start position -> (load arr -> push size chunk pos -> extract
                // u64) -> extract3
                let var_access_chunks = vec![
                    // load arr_index
                    load_chunk.clone(),
                    // pushint pos+16
                    skip_data_chunk,
                    // load arr_index
                    load_chunk,
                    // pushint pos+8
                    skip_size_chunk,
                    // extract_uint64
                    extract_int_chunk,
                    // extract 3 (arr )
                    Chunk::new_empty(Instruction::Extract3),
                ];

                args.concrete_vars.insert(*var_n, var_access_chunks);
            } else {
                let skip_size: u64 = sizes.iter().take(i).sum();
                let take_size = sizes[i];
                let var_access_chunks = vec![
                    load_chunk,
                    Chunk::new_multiple(
                        Instruction::Extract,
                        vec![Constant::Uint(skip_size), Constant::Uint(take_size)],
                    ),
                ];

                args.concrete_vars.insert(*var_n, var_access_chunks);
            }
        }

        let mut bounds_chunks = vec![];
        let mut err = false;
        for e in &bounds.exprs {
            err |= emit_expression(e, &mut bounds_chunks, args).is_err();
            bounds_chunks.push(Chunk::new_empty(Instruction::Assert));
        }

        if err {
            args.diagnostics.push(Report::ver_error(
                s.loc.clone(),
                "Error bounds in initialisation of a model.".to_string(),
            ));
            return Err(());
        }

        chunks.extend(bounds_chunks);

        chunks.push(Chunk::new_single(
            Instruction::Load,
            Constant::Uint(arr_index),
        ));
    }
    Ok(())
}

fn emit_init_args(
    s: &StructInit,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<Vec<u64>, ()> {
    let mut args_sizes = vec![];
    let mut error = false;
    let mut local_chunks = vec![];
    for e in &s.args {
        if let Ok(s) = emit_expression(e, &mut local_chunks, args) {
            args_sizes.push(s);
        } else {
            error |= true;
        }
    }

    if error {
        args.diagnostics.push(Report::ver_error(
            s.loc.clone(),
            "Error evaluating args".to_string(),
        ));
        return Err(());
    }

    local_chunks = vec![];

    let mut start_i = 0;
    let mut storage_sizes = vec![];
    for (e, s) in s.args.iter().zip(&args_sizes) {
        // actual size of the field.
        let mut size = *s;

        // add dynamic sizing for the resizable types.
        if e.ty().is_resizable() {
            let capacity_chunk = Chunk::new_single(Instruction::PushInt, Constant::Uint(*s * 2));
            let size_chunk = Chunk::new_single(Instruction::PushInt, Constant::Uint(*s));
            let itob = Chunk::new_empty(Instruction::Itob);

            // first push capacity chunk
            local_chunks.push(capacity_chunk);
            local_chunks.push(itob.clone());
            local_chunks.push(Chunk::new_single(
                Instruction::Replace,
                Constant::Uint(start_i),
            ));
            start_i += 8;

            // then push size chunk
            local_chunks.push(size_chunk);
            local_chunks.push(itob);
            local_chunks.push(Chunk::new_single(
                Instruction::Replace,
                Constant::Uint(start_i),
            ));
            start_i += 8;

            // the actual size of the field has increased twice + 16 bytes.
            size = (size * 2) + 16;
        }

        // push expression.
        let _ = emit_expression(e, &mut local_chunks, args).expect("should be valid");
        local_chunks.push(Chunk::new_single(
            Instruction::Replace,
            Constant::Uint(start_i),
        ));
        start_i += s;

        storage_sizes.push(size);
    }

    local_chunks.insert(
        0,
        Chunk::new_single(
            Instruction::ArrayInit,
            Constant::Uint(storage_sizes.iter().sum()),
        ),
    );

    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);
    Ok(storage_sizes)
}

fn list(
    u: &UnaryExpression<Vec<Expression>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    if u.element.is_empty() {
        chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(0)));
        chunks.push(Chunk::new_empty(Instruction::ArrayInit));
        return Ok(0);
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

fn func_call(
    f: &FunctionCall,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let mut arg_chunks: Vec<Chunk> = vec![];

    let mut error = false;
    for e in &f.args {
        error |= emit_expression(e, &mut arg_chunks, args).is_err();
    }

    if error {
        return Err(());
    }

    let func_sym = args
        .emitter
        .definition
        .declaration_symbols
        .get(&f.name.name)
        .expect("should exisy")
        .symbol_info();

    let size = &args
        .emitter
        .func_infos
        .get(func_sym)
        .expect("Should exist")
        .return_size;

    chunks.extend(arg_chunks);

    // we use `__<name>` convention for function names.
    let name = format!("__{}", f.name.name);
    chunks.push(Chunk::new_single(
        Instruction::CallSub,
        Constant::StringLit(name),
    ));

    Ok(*size)
}

fn add(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
    // `left + right` should appear in stack as: `left => right => +`

    let mut local_chunks = vec![];
    let s1 = emit_expression(&b.left, &mut local_chunks, args)?;
    let s2 = emit_expression(&b.right, &mut local_chunks, args)?;

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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(s1.max(s2))
}

fn sub(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
    // `left - right` should appear in stack as: `left => right => -`

    let mut local_chunks = vec![];
    let s1 = emit_expression(&b.left, &mut local_chunks, args)?;
    let s2 = emit_expression(&b.right, &mut local_chunks, args)?;

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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(s1.max(s2))
}

fn mul(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
    // `left * right` should appear in stack as: `left => right => *`

    let mut local_chunks = vec![];
    let s1 = emit_expression(&b.left, &mut local_chunks, args)?;
    let s2 = emit_expression(&b.right, &mut local_chunks, args)?;

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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(s1.max(s2))
}

fn div(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
    // `left / right` should appear in stack as: `left => right => /`

    let mut local_chunks = vec![];
    let s1 = emit_expression(&b.left, &mut local_chunks, args)?;
    let s2 = emit_expression(&b.right, &mut local_chunks, args)?;

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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(s1.max(s2))
}

fn modulo(
    b: &BinaryExpression,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let mut local_chunks = vec![];
    let s1 = emit_expression(&b.left, &mut local_chunks, args)?;
    let s2 = emit_expression(&b.right, &mut local_chunks, args)?;

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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(s1.max(s2))
}

fn le(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn leq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn ge(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn geq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn eq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
    // `left == right` should appear in stack as: `left => right => ==`
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    local_chunks.push(Chunk::new_empty(Instruction::BEq));
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn neq(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
    // `left != right` should appear in stack as: `left => right => !=`
    let mut local_chunks = vec![];
    emit_expression(&b.left, &mut local_chunks, args)?;
    emit_expression(&b.right, &mut local_chunks, args)?;

    local_chunks.push(Chunk::new_empty(Instruction::BNeq));
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn not(
    u: &UnaryExpression<Box<Expression>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn or(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

fn and(b: &BinaryExpression, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> Result<u64, ()> {
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
    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(8)
}

/// Load var from the scratch.
fn var(
    u: &UnaryExpression<usize>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let Some(var) = args.scratch.get_var(u.element) else {
        args.diagnostics.push(Report::emit_error(
            u.loc.clone(),
            String::from("Variable does not exist."),
        ));
        return Err(());
    };

    if let Some(local_chunks) = args.concrete_vars.get(&u.element) {
        chunks.extend_from_slice(local_chunks);
        return Ok(0);
    }

    let c = Constant::Uint(var.index as u64);
    let chunk = Chunk::new_single(Instruction::Load, c);

    chunks.push(chunk);

    Ok(var.size)
}

/// Handle signed and unsigned integers as bytes.
fn int(
    n: &BigInt,
    loc: &Span,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
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

    let len = bytes.len() as u64;
    let c = Constant::Bytes(bytes);
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(len)
}

/// Handle boolean values as `1` and `0` in Teal.
fn bool(
    u: &UnaryExpression<bool>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let val: u64 = if u.element { 1 } else { 0 };
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(8)
}

/// Handle character as u64 value.
fn char(
    u: &UnaryExpression<char>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let val: u64 = u.element.into();
    let c = Constant::Uint(val);
    let chunk = Chunk::new_single(Instruction::PushInt, c);
    chunks.push(chunk);

    Ok(8)
}

/// Handle raw string literals.
fn string(
    u: &UnaryExpression<String>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let c = Constant::String(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(u.element.len() as u64)
}

/// Handle hex value bytes.
fn hex(
    u: &UnaryExpression<Vec<u8>>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let c = Constant::Bytes(u.element.clone());
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(u.element.len() as u64)
}

/// Handle Algorand address.
fn address(
    u: &UnaryExpression<Address>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let c = Constant::StringLit(u.element.to_string());
    let chunk = Chunk::new_single(Instruction::PushAddr, c);
    chunks.push(chunk);

    Ok(32)
}

/// Handle enum literals, we construct it from `bytes(enums_name) + byte(variant_number)`
fn enum_(
    u: &UnaryExpression<usize>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
    let TypeVariant::Enum(s) = &u.ty else {
        unreachable!()
    };

    let mut enum_name = args.emitter.definition.enums[s.i]
        .name
        .name
        .clone()
        .as_bytes()
        .to_vec();
    enum_name.push(u.element as u8);

    let len = enum_name.len() as u64;
    let c = Constant::Bytes(enum_name);
    let chunk = Chunk::new_single(Instruction::PushBytes, c);
    chunks.push(chunk);

    Ok(len)
}

/// Handle rational literals, for now we simply present them at f64 IEEE 754 standard.
fn float(
    u: &UnaryExpression<BigRational>,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<u64, ()> {
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

    Ok(8)
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

fn add_padding(chunks: &mut Vec<Chunk>) {
    chunks.insert(0, Chunk::new_empty(Instruction::Empty));
    chunks.push(Chunk::new_empty(Instruction::Empty));
}
