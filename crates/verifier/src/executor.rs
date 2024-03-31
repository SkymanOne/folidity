use std::rc::Rc;

use folidity_diagnostics::Report;
use folidity_semantics::{
    ContractDefinition,
    GlobalSymbol,
    SymbolInfo,
};
use z3::{
    ast::{
        Ast,
        Dynamic,
    },
    Context,
    Solver,
    Sort,
};

use crate::{
    ast::{
        Constraint,
        Declaration,
        Z3Expression,
    },
    transformer::transform_expr,
    z3_cfg,
};

//

#[derive(Debug)]
pub struct SymbolicExecutor<'ctx> {
    /// Global solver of the executor.
    ///
    /// We encapsulate it as it can't be easily transferred between scopes.
    context: Context,
    /// List of resolved declaration to verify.
    pub declarations: Vec<Declaration<'ctx>>,
    pub contexts: Vec<Context>,
    /// Symbol counter to track boolean constants across the program.
    pub symbol_counter: u32,
    pub diagnostics: Vec<Report>,
}

impl<'ctx> SymbolicExecutor<'ctx> {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            contexts: vec![],
            declarations: vec![],
            diagnostics: vec![],
            symbol_counter: 0,
        }
    }

    pub fn parse_declarations(&mut self, contract: &ContractDefinition) -> Result<(), ()> {
        let mut error = false;
        let mut diagnostics = Vec::new();

        for (i, m) in contract.models.iter().enumerate() {
            let context = Context::new(&z3_cfg());
            let z3_exprs: Vec<Z3Expression> = m
                .bounds
                .iter()
                .filter_map(|e| {
                    match transform_expr(e, &context, &mut diagnostics, self) {
                        Ok(c) => Some(c),
                        Err(_) => {
                            error = true;
                            None
                        }
                    }
                })
                .collect();

            // let decl = Declaration {
            //     decl_sym: GlobalSymbol::Model(SymbolInfo::new(m.loc.clone(), i)),
            //     parent: m.parent.clone(),
            //     constraints,
            // };

            // self.declarations.push(decl);
        }

        if error {
            return Err(());
        }
        Ok(())
    }

    /// Create a Z3 constant with the current symbol counter as a name while increasing
    /// the counter.
    pub fn create_constant<'a>(
        &mut self,
        sort: &Sort<'a>,
        context: &'a Context,
    ) -> (Dynamic<'a>, u32) {
        let id = self.symbol_counter;
        let c = Dynamic::new_const(&context, id, sort);
        self.symbol_counter += 1;
        (c, id)
    }

    /// Retrieve the context of the internal `solver`.
    pub fn context(&self) -> &Context {
        &self.context
    }
}
