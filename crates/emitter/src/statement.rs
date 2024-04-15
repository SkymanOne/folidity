use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::ast::{
    Assign,
    Expression,
    ForLoop,
    FuncReturnType,
    IfElse,
    Statement,
    TypeVariant,
    Variable,
};

use crate::{
    add_padding,
    ast::{
        Chunk,
        Constant,
        Instruction,
    },
    expression::emit_expression,
    teal::EmitArgs,
};

type EmitResult = Result<(), ()>;

pub fn emit_statement(
    stmt: &Statement,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
    match stmt {
        Statement::Variable(var) => variable(var, stmt.loc(), chunks, args),
        Statement::Assign(var) => assign(var, stmt.loc(), chunks, args),
        Statement::Expression(e) => emit_expression(e, chunks, args).map(|_| ()),
        Statement::IfElse(b) => if_else(b, chunks, args),
        Statement::ForLoop(l) => for_loop(l, chunks, args),
        Statement::Iterator(_) => todo!("Not yet implemented in emitter"),
        Statement::Return(r) => return_(&r.expr, chunks, args),
        Statement::StateTransition(e) => state_transition(e, chunks, args),
        Statement::Block(b) => block(&b.statements, chunks, args),
        Statement::Skip(loc) => skip(loc, chunks, args),
        Statement::Error(_) => unreachable!(),
    }
}

fn variable(
    var: &Variable,
    loc: &Span,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitArgs,
) -> EmitResult {
    // todo: destructure fields.
    if var.names.len() != 1 {
        args.diagnostics.push(Report::ver_error(
            loc.clone(),
            String::from("Destructuring is currently unsupported."),
        ));
        return Err(());
    }
    let mut size = 0;

    if let Some(e) = &var.value {
        size = emit_expression(e, chunks, args)?
    } else {
        chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(0)));
        chunks.push(Chunk::new_empty(Instruction::ArrayInit));
    }

    let index = args.scratch.add_var(var.pos, size, args.emitter) as u64;
    chunks.push(Chunk::new_single(
        Instruction::PushInt,
        Constant::Uint(index),
    ));
    chunks.push(Chunk::new_empty(Instruction::Store));

    Ok(())
}

fn assign(var: &Assign, loc: &Span, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let size = emit_expression(&var.value, chunks, args)?;
    let Some(var_scratch) = args.scratch.get_var_mut(var.pos) else {
        args.diagnostics.push(Report::ver_error(
            loc.clone(),
            String::from("Variable is undeclared."),
        ));
        return Err(());
    };

    var_scratch.size = size;
    let index = var_scratch.index as u64;
    chunks.push(Chunk::new_single(
        Instruction::PushInt,
        Constant::Uint(index),
    ));
    chunks.push(Chunk::new_empty(Instruction::Store));

    Ok(())
}

fn skip(loc: &Span, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    if args.loop_labels.is_empty() {
        args.diagnostics.push(Report::ver_error(
            loc.clone(),
            String::from("Not a loop context."),
        ));
    }

    chunks.push(Chunk::new_single(
        Instruction::Branch,
        Constant::String(args.loop_labels.last().expect("should exist").clone()),
    ));

    Ok(())
}

fn block(stmts: &[Statement], chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut error = false;

    for stmt in stmts {
        error |= emit_statement(stmt, chunks, args).is_err();
    }

    if error {
        return Err(());
    }

    Ok(())
}

fn for_loop(l: &ForLoop, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut loop_chunks = vec![];
    let loop_index = args.emitter.loop_index_incr()?;
    let start_label = format!("{}_loop_start", loop_index);
    let end_label = format!("{}_loop_end", loop_index);
    let mut error = false;

    // create var and store it.
    error |= variable(&l.var, &l.loc, &mut loop_chunks, args).is_err();

    // create start label and push it to the top of the stack.
    loop_chunks.push(Chunk::new_empty(Instruction::Label(start_label.clone())));
    // emit condition block
    error |= emit_expression(&l.condition, &mut loop_chunks, args).is_err();
    // jump to end if satisfies (i.e. 1)
    loop_chunks.push(Chunk::new_single(
        Instruction::BranchNotZero,
        Constant::String(end_label.clone()),
    ));

    // emit body
    args.loop_labels.push(start_label.clone());
    error |= block(&l.body, &mut loop_chunks, args).is_err();

    // emit increment logic
    error |= emit_expression(&l.incrementer, &mut loop_chunks, args).is_err();

    // emit end label.
    loop_chunks.push(Chunk::new_empty(Instruction::Label(end_label.clone())));

    // pop label
    args.loop_labels.pop();

    // pad the block.
    add_padding(&mut loop_chunks);

    if error {
        return Err(());
    }

    chunks.extend(loop_chunks);

    Ok(())
}

fn if_else(b: &IfElse, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut block_chunks = vec![];
    let index = args.emitter.cond_index_incr()?;
    let else_label = format!("{}_else", index);
    let end_label = format!("{}_if_end", index);
    let mut error = false;

    // emit condition block
    error |= emit_expression(&b.condition, &mut block_chunks, args).is_err();
    // jump to else block if not satisfies (i.e. 0)
    block_chunks.push(Chunk::new_single(
        Instruction::BranchZero,
        Constant::String(else_label.clone()),
    ));

    // emit body.
    error |= block(&b.body, &mut block_chunks, args).is_err();
    // jump to end if executed.
    block_chunks.push(Chunk::new_single(
        Instruction::Branch,
        Constant::String(end_label.clone()),
    ));

    // emit else parts
    // create else label and push it to the top of the stack.
    block_chunks.push(Chunk::new_empty(Instruction::Label(else_label.clone())));
    error |= block(&b.else_part, &mut block_chunks, args).is_err();

    // create end label and push it to the top of the stack.
    block_chunks.push(Chunk::new_empty(Instruction::Label(end_label.clone())));

    // pad the block.
    add_padding(&mut block_chunks);

    if error {
        return Err(());
    }

    chunks.extend(block_chunks);

    Ok(())
}

fn state_transition(e: &Expression, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let mut local_chunks = vec![];

    let TypeVariant::State(sym) = e.ty() else {
        unreachable!()
    };
    let state_decl = &args.emitter.definition.states[sym.i];
    let box_name = format!("__{}", state_decl.name.name);

    // push name of a box onto stack
    let name_chunk = Chunk::new_single(Instruction::PushBytes, Constant::String(box_name));
    local_chunks.push(name_chunk);

    // push contents onto the stack
    let _ = emit_expression(e, &mut local_chunks, args)?;

    if let Some(state_bound) = &args.func.state_bound {
        let TypeVariant::State(sym) = e.ty() else {
            unreachable!();
        };

        if let Some(param) = state_bound.to.iter().find(|s| &s.ty == sym) {
            if let Some(name) = &param.name {
                let (p_no, _) = args
                    .func
                    .scope
                    .find_var_index(&name.name)
                    .expect("should exist");
                // saves contents in the scratch.
                let index = args.emitter.scratch_index_incr()? as u64;
                local_chunks.push(Chunk::new_single(Instruction::Store, Constant::Uint(index)));

                args.concrete_vars.insert(
                    p_no,
                    vec![Chunk::new_single(Instruction::Load, Constant::Uint(index))],
                );

                emit_bounds(&mut local_chunks, args);

                // load contents from the scratch.
                local_chunks.push(Chunk::new_single(Instruction::Load, Constant::Uint(index)));
                // reset index
                args.emitter.scratch_index = index as u8;
            }
        }
    }

    // todo: support sizes of >4096 bytes
    // push `box_put` onto the stack which creates a box or replaces contents of the existing
    // one.
    let box_chunk = Chunk::new_empty(Instruction::BoxPut);
    local_chunks.push(box_chunk);

    add_padding(&mut local_chunks);
    chunks.extend(local_chunks);

    Ok(())
}

fn return_(e: &Option<Expression>, chunks: &mut Vec<Chunk>, args: &mut EmitArgs) -> EmitResult {
    let Some(expr) = e else {
        chunks.push(Chunk::new_empty(Instruction::ReturnSubroutine));
        return Ok(());
    };

    let mut local_chunks = vec![];
    let _ = emit_expression(expr, &mut local_chunks, args)?;

    if let FuncReturnType::ParamType(param) = &args.func.return_ty {
        let index = args.emitter.scratch_index_incr()? as u64;
        local_chunks.push(Chunk::new_single(Instruction::Store, Constant::Uint(index)));

        let (p_no, _) = args
            .func
            .scope
            .find_var_index(&param.name.name)
            .expect("should exist");

        args.concrete_vars.insert(
            p_no,
            vec![Chunk::new_single(Instruction::Load, Constant::Uint(index))],
        );

        emit_bounds(&mut local_chunks, args);
        local_chunks.push(Chunk::new_single(Instruction::Load, Constant::Uint(index)));

        // reset index
        args.emitter.scratch_index = index as u8;
    }

    chunks.push(Chunk::new_empty(Instruction::ReturnSubroutine));

    Ok(())
}

pub fn emit_bounds(chunks: &mut Vec<Chunk>, args: &mut EmitArgs) {
    let mut delayed_bounds = vec![];
    std::mem::swap(args.delayed_bounds, &mut delayed_bounds);

    let mut bound_chunks = vec![];

    // save diagnostics state
    let mut diagnostics = vec![];
    std::mem::swap(args.diagnostics, &mut diagnostics);

    while let Some(e) = delayed_bounds.last() {
        // if expression has been successfully emitter, we can pop it from the list.
        if emit_expression(e, &mut bound_chunks, args).is_ok() {
            // and assert it
            bound_chunks.push(Chunk::new_empty(Instruction::Assert));
            delayed_bounds.pop();
        }
    }

    std::mem::swap(args.delayed_bounds, &mut delayed_bounds);
    // recover the state.
    std::mem::swap(args.diagnostics, &mut diagnostics);

    add_padding(&mut bound_chunks);
    chunks.extend(bound_chunks);
}
