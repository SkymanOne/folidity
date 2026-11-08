use folidity_diagnostics::Report;
use folidity_semantics::{
    ast::{
        Expression,
        Function,
        FunctionVisibility,
        TypeVariant,
    },
    ContractDefinition,
    Span,
};
use indexmap::IndexMap;

use crate::{
    add_padding,
    ast::{
        Chunk,
        Constant,
        Instruction,
    },
    function::emit_function,
    scratch_table::ScratchTable,
};

/// Arguments for emitter operations.
#[derive(Debug)]
pub struct EmitArgs<'a, 'b> {
    pub scratch: &'b mut ScratchTable,
    pub diagnostics: &'b mut Vec<Report>,
    pub emitter: &'b mut TealEmitter<'a>,
    pub delayed_bounds: &'b mut Vec<Expression>,
    pub func: &'b Function,
    pub loop_labels: &'b mut Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TealArtifacts {
    /// Teal approval program bytes.
    pub approval_bytes: Vec<u8>,
    /// Teal clear program bytes.
    pub clear_bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct TealEmitter<'a> {
    /// Nested definition of the contract.
    pub definition: &'a ContractDefinition,
    /// List of chunks that are emitted into the final build.
    chunks: Vec<Chunk>,
    /// Errors and warning caused during emit process.
    pub diagnostics: Vec<Report>,
    /// Index for scratch space variable.
    ///
    /// We use `u8` as there are only 256 cells available.
    pub scratch_index: u8,

    /// Counter for loops.
    pub loop_counter: u64,

    /// Counter for if-else.
    pub cond_counter: u64,
    /// list of concrete teal expression to access vars.
    pub concrete_vars: IndexMap<usize, Vec<Chunk>>,
}

impl<'a> TealEmitter<'a> {
    pub fn new(definition: &'a ContractDefinition) -> Self {
        Self {
            definition,
            chunks: vec![],
            diagnostics: vec![],
            // 0 and 1 are reserved for signed_arithmetic.teal helpers.
            scratch_index: 2,
            loop_counter: 0,
            cond_counter: 0,
            concrete_vars: IndexMap::new(),
        }
    }

    pub fn emit_entry_point(&mut self) {
        let mut chunks = vec![];
        let create_end_label = "create_end".to_string();

        chunks.push(Chunk::new_single(
            Instruction::Txn,
            Constant::StringLit("ApplicationID".to_string()),
        ));
        chunks.push(Chunk::new_single(Instruction::PushInt, Constant::Uint(0)));
        chunks.push(Chunk::new_empty(Instruction::Eq));
        chunks.push(Chunk::new_single(
            Instruction::BranchZero,
            Constant::StringLit(create_end_label.clone()),
        ));

        let init_func_i = self
            .definition
            .functions
            .iter()
            .position(|f| f.is_init)
            .expect("should be defined");

        let init_func_name = format!(
            "__block__{}",
            self.definition.functions[init_func_i].name.name
        );
        chunks.push(Chunk::new_single(
            Instruction::Branch,
            Constant::StringLit(init_func_name),
        ));
        chunks.push(Chunk::new_empty(Instruction::Label(create_end_label)));

        chunks.extend_from_slice(&[
            Chunk::new_single(
                Instruction::Txn,
                Constant::StringLit("OnCompletion".to_string()),
            ),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(0)), // NoOp
            Chunk::new_empty(Instruction::Eq),
            Chunk::new_single(
                Instruction::BranchNotZero,
                Constant::StringLit("on_call".to_string()),
            ),
            Chunk::new_single(
                Instruction::Txn,
                Constant::StringLit("OnCompletion".to_string()),
            ),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(5)), // Delete
            Chunk::new_empty(Instruction::Eq),
            Chunk::new_single(
                Instruction::BranchNotZero,
                Constant::StringLit("check_creator".to_string()),
            ),
            Chunk::new_single(
                Instruction::Txn,
                Constant::StringLit("OnCompletion".to_string()),
            ),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(1)), // OptIn
            Chunk::new_empty(Instruction::Eq),
            Chunk::new_single(
                Instruction::BranchNotZero,
                Constant::StringLit("fail".to_string()), // fail for now
            ),
            Chunk::new_single(
                Instruction::Txn,
                Constant::StringLit("OnCompletion".to_string()),
            ),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(2)), // CloseOut
            Chunk::new_empty(Instruction::Eq),
            Chunk::new_single(
                Instruction::BranchNotZero,
                Constant::StringLit("fail".to_string()), // fail for now
            ),
            Chunk::new_single(
                Instruction::Txn,
                Constant::StringLit("OnCompletion".to_string()),
            ),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(4)), // UpdateApplication
            Chunk::new_empty(Instruction::Eq),
            Chunk::new_single(
                Instruction::BranchNotZero,
                Constant::StringLit("check_creator".to_string()),
            ),
            Chunk::new_empty(Instruction::Error), // error if None matches.
            //
            Chunk::new_empty(Instruction::Empty),
            Chunk::new_empty(Instruction::Empty),
            //
            Chunk::new_empty(Instruction::Label("check_creator".to_string())),
            Chunk::new_single(Instruction::Txn, Constant::StringLit("Sender".to_string())),
            Chunk::new_single(
                Instruction::Global,
                Constant::StringLit("CreatorAddress".to_string()),
            ),
            Chunk::new_empty(Instruction::Eq),
            Chunk::new_empty(Instruction::Assert),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(1)),
            Chunk::new_empty(Instruction::Return),
            //
            Chunk::new_empty(Instruction::Empty),
            Chunk::new_empty(Instruction::Empty),
        ]);

        // return 0 error code.
        chunks.extend_from_slice(&[
            Chunk::new_empty(Instruction::Label("fail".to_string())),
            Chunk::new_single(Instruction::PushInt, Constant::Uint(0)),
            Chunk::new_empty(Instruction::Return),
            Chunk::new_empty(Instruction::Empty),
            Chunk::new_empty(Instruction::Empty),
        ]);

        chunks.push(Chunk::new_empty(Instruction::Label("on_call".to_string())));

        for f in &self.definition.functions {
            if f.is_init || matches!(f.vis, FunctionVisibility::Priv) {
                continue;
            }
            let name = &f.name.name;
            chunks.extend_from_slice(&[
                Chunk::new_multiple(
                    Instruction::Txna,
                    vec![
                        Constant::StringLit("ApplicationArgs".to_string()),
                        Constant::Uint(0),
                    ],
                ),
                Chunk::new_single(Instruction::PushBytes, Constant::String(name.clone())),
                Chunk::new_empty(Instruction::Eq),
                Chunk::new_single(
                    Instruction::BranchNotZero,
                    Constant::StringLit(format!("__block__{}", name)),
                ),
            ]);
        }
        chunks.push(Chunk::new_empty(Instruction::Error)); // error if none matches.

        let mut block_chunks = self.emit_blocks();
        add_padding(&mut block_chunks);
        chunks.extend(block_chunks);

        add_padding(&mut chunks);
        self.chunks.extend(chunks);
    }

    pub fn emit_functions(&mut self) -> bool {
        let mut error = false;

        for func in &self.definition.functions {
            if let Ok(mut chunks) = emit_function(func, self) {
                add_padding(&mut chunks);
                self.chunks.extend(chunks);
            } else {
                error |= true;
            }
        }

        !error
    }

    pub fn compile(&mut self) -> TealArtifacts {
        let approval_string = self
            .chunks
            .iter()
            .fold("#pragma version 8".to_string(), |init, c| {
                format!("{}\n{}", init, c)
            });
        let mut approval_bytes: Vec<u8> = approval_string.bytes().collect();

        let clear_chunks = [
            Chunk::new_single(Instruction::PushInt, Constant::Uint(0)),
            Chunk::new_empty(Instruction::Return),
        ];
        let clear_string = clear_chunks
            .iter()
            .fold("#pragma version 8".to_string(), |init, c| {
                format!("{}\n{}", init, c)
            });
        let clear_bytes: Vec<u8> = clear_string.bytes().collect();

        let helper_bytes = include_bytes!("../helpers/signed_arithmetic.teal");
        approval_bytes.extend_from_slice(helper_bytes);

        TealArtifacts {
            approval_bytes,
            clear_bytes,
        }
    }

    #[allow(clippy::result_unit_err)]
    pub fn scratch_index_incr(&mut self) -> Result<u64, ()> {
        let i = self.scratch_index;
        self.scratch_index = self.scratch_index.checked_add(1).ok_or_else(|| {
            self.diagnostics.push(Report::emit_error(
                Span::default(),
                "Exceeded variable count".to_string(),
            ))
        })?;

        Ok(i as u64)
    }

    #[allow(clippy::result_unit_err)]
    pub fn loop_index_incr(&mut self) -> Result<u64, ()> {
        let i = self.loop_counter;
        self.loop_counter = self.loop_counter.checked_add(1).ok_or_else(|| {
            self.diagnostics.push(Report::emit_error(
                Span::default(),
                "Exceeded loop count".to_string(),
            ))
        })?;

        Ok(i)
    }

    #[allow(clippy::result_unit_err)]
    pub fn cond_index_incr(&mut self) -> Result<u64, ()> {
        let i = self.cond_counter;
        self.cond_counter = self.cond_counter.checked_add(1).ok_or_else(|| {
            self.diagnostics.push(Report::emit_error(
                Span::default(),
                "Exceeded if-else count".to_string(),
            ))
        })?;

        Ok(i)
    }

    fn emit_blocks(&mut self) -> Vec<Chunk> {
        let mut chunks = vec![];

        for i in 0..self.definition.functions.len() {
            let f = &self.definition.functions[i];
            if matches!(f.vis, FunctionVisibility::Priv) && !f.is_init {
                continue;
            }
            let is_init = f.is_init;
            let name = f.name.name.clone();
            let param_tys: Vec<TypeVariant> = f.params.values().map(|p| p.ty.ty.clone()).collect();
            let returns_unit = f.return_ty.ty() == &TypeVariant::Unit;

            let mut block_chunks = vec![];
            let block_name = format!("__block__{}", name);
            let func_name = format!("__{}", name);

            block_chunks.push(Chunk::new_empty(Instruction::Label(block_name)));

            // push argument into the function block.
            // if the function is not a constructor, then the first app arg is a function
            // signature.
            let mut func_arg_index: u64 = if is_init { 0 } else { 1 };
            for ty in &param_tys {
                block_chunks.push(Chunk::new_multiple(
                    Instruction::Txna,
                    vec![
                        Constant::StringLit("ApplicationArgs".to_string()),
                        Constant::Uint(func_arg_index),
                    ],
                ));
                decode_app_arg(ty, &mut block_chunks, self);
                func_arg_index += 1;
            }

            block_chunks.push(Chunk::new_single(
                Instruction::CallSub,
                crate::ast::Constant::StringLit(func_name),
            ));

            if !returns_unit {
                block_chunks.push(Chunk::new_empty(Instruction::Log));
            }

            block_chunks.extend_from_slice(&[
                Chunk::new_single(Instruction::PushInt, Constant::Uint(1)),
                Chunk::new_empty(Instruction::Return),
            ]);

            add_padding(&mut block_chunks);
            chunks.extend(block_chunks);
        }
        chunks
    }
}

fn decode_app_arg(ty: &TypeVariant, chunks: &mut Vec<Chunk>, emitter: &mut TealEmitter) {
    match ty {
        TypeVariant::Uint | TypeVariant::Bool | TypeVariant::Char | TypeVariant::Float => {
            chunks.push(Chunk::new_empty(Instruction::Btoi));
        }
        TypeVariant::Int => {
            chunks.push(Chunk::new_empty(Instruction::Btoi));
            let Ok(tmp) = emitter.scratch_index_incr() else {
                return;
            };
            chunks.extend_from_slice(&[
                Chunk::new_single(Instruction::Store, Constant::Uint(tmp)),
                Chunk::new_single(Instruction::PushInt, Constant::Uint(16)),
                Chunk::new_empty(Instruction::ArrayInit),
                Chunk::new_single(Instruction::Load, Constant::Uint(tmp)),
                Chunk::new_single(Instruction::Replace, Constant::Uint(8)),
            ]);
        }
        _ => {}
    }
}
