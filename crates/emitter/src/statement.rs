use folidity_diagnostics::{
    Report,
    Span,
};
use folidity_semantics::ast::{
    Assign,
    ForLoop,
    IfElse,
    Statement,
    Variable,
};

use crate::{
    add_padding,
    ast::{
        Chunk,
        Constant,
        Instruction,
    },
    expression::{
        emit_expression,
        EmitExprArgs,
    },
};

type EmitResult = Result<(), ()>;

pub fn emit_statement(
    stmt: &Statement,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> EmitResult {
    match stmt {
        Statement::Variable(var) => variable(var, stmt.loc(), chunks, args),
        Statement::Assign(var) => assign(var, stmt.loc(), chunks, args),
        Statement::Expression(e) => emit_expression(e, chunks, args).map(|_| ()),
        Statement::IfElse(b) => if_else(b, chunks, args),
        Statement::ForLoop(l) => for_loop(l, chunks, args),
        Statement::Iterator(_) => todo!(),
        Statement::Return(_) => todo!(),
        Statement::StateTransition(_) => todo!(),
        Statement::Block(b) => block(&b.statements, chunks, args),
        Statement::Skip(loc) => skip(loc, chunks, args),
        Statement::Error(_) => unreachable!(),
    }
}

fn variable(
    var: &Variable,
    loc: &Span,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
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

fn assign(
    var: &Assign,
    loc: &Span,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> EmitResult {
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

fn skip(loc: &Span, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> EmitResult {
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

fn block(stmts: &[Statement], chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> EmitResult {
    let mut error = false;

    for stmt in stmts {
        error |= emit_statement(stmt, chunks, args).is_err();
    }

    if error {
        return Err(());
    }

    Ok(())
}

fn for_loop(l: &ForLoop, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> EmitResult {
    let mut loop_chunks = vec![];
    let loop_index = args.emitter.loop_index_incr()?;
    let start_label = format!("{}_loop_start", loop_index);
    let end_label = format!("{}_loop_end", loop_index);
    let mut error = false;

    // create var and store it.
    error |= variable(&l.var, &l.loc, &mut loop_chunks, args).is_err();

    // create label and push it to the top of the stack.
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

fn if_else(b: &IfElse, chunks: &mut Vec<Chunk>, args: &mut EmitExprArgs) -> EmitResult {
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
