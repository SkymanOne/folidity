use folidity_diagnostics::Report;
use folidity_semantics::{
    ast::{
        Function,
        FunctionVisibility,
        TypeVariant,
    },
    SymbolInfo,
};
use indexmap::IndexMap;

use crate::{
    ast::{
        Chunk,
        Constant,
        Instruction,
    },
    expression::emit_expression,
    scratch_table::ScratchTable,
    statement::{
        emit_bounds,
        emit_statement,
    },
    teal::{
        EmitArgs,
        TealEmitter,
    },
};

pub fn emit_function(func: &Function, emitter: &mut TealEmitter) -> Result<Vec<Chunk>, ()> {
    let mut chunks = vec![];
    let func_name = format!("__{}", func.name.name);
    chunks.push(Chunk::new_empty(Instruction::Label(func_name.clone())));

    let mut error = false;
    let mut scratch = ScratchTable::default();
    let mut concrete_vars = IndexMap::new();
    let mut diagnostics = vec![];
    let mut args = EmitArgs {
        scratch: &mut scratch,
        diagnostics: &mut diagnostics,
        delayed_bounds: &mut vec![],
        emitter,
        concrete_vars: &mut concrete_vars,
        func,
        loop_labels: &mut vec![],
    };

    // inject arguments as concrete vars.
    // if the function is not a constructor, then the first app arg is a function signature.
    let mut func_arg_index: u64 = if func.is_init { 0 } else { 1 };
    for (name, _) in &func.params {
        let (p_no, _) = func.scope.find_var_index(name).expect("should exist");
        let arg_chunk = Chunk::new_multiple(
            Instruction::Txn,
            vec![
                Constant::StringLit("ApplicationArgs".to_string()),
                Constant::Uint(func_arg_index),
            ],
        );
        args.concrete_vars.insert(p_no, vec![arg_chunk]);

        func_arg_index += 1;
    }

    // Inject concrete vars for state bounds.
    if let Some(bounds) = &func.state_bound {
        // if there is from bound, then we assume that the box has been created.
        if let Some(from) = &bounds.from {
            if let Some(var_ident) = &from.name {
                emit_state_var(&var_ident.name, &from.ty, func, &mut args);
            }
        }
    }

    // emit access check chunks.
    let mut access_chunks = vec![];
    for e in &func.access_attributes {
        if e.ty() != &TypeVariant::Address {
            diagnostics.push(Report::emit_error(
                e.loc().clone(),
                "Non-address types are currently unsupported in emitter".to_string(),
            ));
            return Err(());
        }

        error |= emit_expression(e, &mut access_chunks, &mut args).is_err();
        chunks.push(Chunk::new_single(
            Instruction::Txn,
            Constant::StringLit("Sender".to_string()),
        ));
    }

    // if view function, emit similar concrete state var.
    if let FunctionVisibility::View(s) = &func.vis {
        emit_state_var(&s.name.name, &s.ty, func, &mut args);
    }

    chunks.extend(access_chunks);

    // emit bound expressions for input state and args
    // any unresolved expression are added to the delay.
    if let Some(bounds) = &func.bounds {
        args.delayed_bounds.extend_from_slice(&bounds.exprs);

        emit_bounds(&mut chunks, &mut args);
    }

    // emit statements.
    let mut body_chunks = vec![];
    for stmt in &func.body {
        error |= emit_statement(stmt, &mut body_chunks, &mut args).is_err();
    }
    chunks.extend(body_chunks);

    chunks.push(Chunk::new_empty(Instruction::ReturnSubroutine));

    if error {
        emitter.diagnostics.extend(diagnostics);
        return Err(());
    }

    Ok(chunks)
}

fn emit_state_var(ident: &str, sym: &SymbolInfo, func: &Function, args: &mut EmitArgs) {
    let state_decl = &args.emitter.definition.states[sym.i];
    let box_name = format!("__{}", state_decl.name.name);
    let (v_no, _) = func.scope.find_var_index(ident).expect("should exist");

    // todo: support sizes of >4096 bytes
    let name_chunk = Chunk::new_single(Instruction::PushBytes, Constant::String(box_name));
    let box_chunk = Chunk::new_empty(Instruction::BoxGet);
    let assert_chunk = Chunk::new_empty(Instruction::Assert);

    args.concrete_vars
        .insert(v_no, vec![name_chunk, box_chunk, assert_chunk]);
}
