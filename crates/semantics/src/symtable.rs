use std::collections::HashMap;

use folidity_parser::ast::Identifier;
use indexmap::IndexMap;

use crate::{
    ast::{
        Expression,
        TypeVariant,
    },
    contract::ContractDefinition,
    global_symbol::GlobalSymbol,
};

#[derive(Debug, Clone)]
pub struct VariableSym {
    /// Name of the variable.
    pub ident: Identifier,
    /// Type of the variable.
    pub ty: TypeVariant,
    /// Assigned value of a variable
    pub value: Option<Expression>,
    /// Has the variable been used?
    pub used: bool,
    /// The usage context of the variable.
    pub usage: VariableKind,
    /// Can the variable be mutated.
    pub mutable: bool,
}

impl VariableSym {
    /// Check if the variable has been assigned an expression.
    pub fn assigned(&self) -> bool {
        self.value.is_some()
    }
}

/// A kind of a variable used.
#[derive(Debug, Clone, PartialEq)]
pub enum VariableKind {
    Destructor,
    Param,
    Local,
    FromState,
    ToState,
    Loop,
    Return,
}

/// Context of the scope in the symtable.
/// Typical structure would be:
/// `FunctionParams` -> `Bounds` -> `FunctionBody` -> ...
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ScopeContext {
    /// We are inside context of declaration of the symbol and its bounds.
    DeclarationBounds,
    /// Scope is in the function with the given index.
    #[default]
    FunctionBody,
    Loop,
    Block,
}

#[derive(Debug, Clone, Default)]
pub struct SymTable {
    /// Variable names in the current scope.
    pub names: HashMap<String, usize>,
    // Context of variables in the given scope.
    pub context: ScopeContext,
}

#[derive(Debug, Clone)]
pub struct Scope {
    /// Indexed map of variables
    pub vars: IndexMap<usize, VariableSym>,
    /// List of scoped symbol tables.
    pub tables: Vec<SymTable>,
    /// Index of the current scope.
    pub current: usize,
    /// What symbol this scope this belongs to.
    pub symbol: GlobalSymbol,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            vars: IndexMap::new(),
            tables: vec![SymTable::default()],
            current: 0,
            symbol: GlobalSymbol::default(),
        }
    }
}

impl Scope {
    pub fn new(sym: &GlobalSymbol, context: ScopeContext) -> Self {
        Self {
            tables: vec![SymTable {
                names: HashMap::new(),
                context,
            }],
            current: 0,
            symbol: sym.clone(),
            vars: Default::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        ident: &Identifier,
        ty: TypeVariant,
        value: Option<Expression>,
        usage: VariableKind,
        mutable: bool,
        table_pos: usize,
        contract: &mut ContractDefinition,
    ) -> usize {
        let current_id = contract.next_var_id;
        contract.next_var_id += 1;

        self.vars.insert(
            current_id,
            VariableSym {
                ident: ident.clone(),
                ty,
                value,
                usage,
                used: false,
                mutable,
            },
        );

        self.tables[table_pos]
            .names
            .insert(ident.name.clone(), current_id);
        current_id
    }

    /// Attempts to find an index of a symbol in the current or outer scopes.
    ///
    /// # Returns
    /// - Index of a symbol in the list of vars.
    /// - Index of the table where the symbol can be found if any.
    pub fn find_var_index(&self, name: &str) -> Option<(usize, usize)> {
        let mut table_i = self.current;
        let mut table = &self.tables[table_i];

        // we need to decide which variables we are allowed traverse depending on the context of
        // the current scope.
        let whitelists = match &table.context {
            // if we are inside bound context, we can only traverse params, access attributes,
            // return param, and state bounds.
            ScopeContext::DeclarationBounds => {
                vec![
                    VariableKind::Local,
                    VariableKind::Param,
                    VariableKind::ToState,
                    VariableKind::FromState,
                    VariableKind::Return,
                ]
            }
            // if we inside loop, block or function body, then we can traverse them, function
            // params, and initial state bound.
            ScopeContext::FunctionBody | ScopeContext::Block | ScopeContext::Loop => {
                vec![
                    VariableKind::FromState,
                    VariableKind::Param,
                    VariableKind::Local,
                    VariableKind::Loop,
                    VariableKind::Destructor,
                ]
            }
        };

        let mut v_i = table.names.get(name);
        while table_i > 0 && v_i.is_none() {
            table_i -= 1;
            table = &self.tables[table_i];
            v_i = table.names.get(name);
        }

        if let Some(i) = v_i {
            let var = self.vars.get(i).unwrap();
            // we only want to return variable that can be access by current scope.
            if whitelists.contains(&var.usage) {
                Some((*i, table_i))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Attempts to find a symbol with the given index in the current or outer scopes.
    ///
    /// # Returns
    /// - A reference to the symbol in the table if any
    pub fn find_symbol(&self, index: &usize) -> Option<&VariableSym> {
        self.vars.get(index)
    }

    /// Pushes the scope context onto the stack.
    pub fn push(&mut self, context: ScopeContext) {
        if self.current == self.tables.len() - 1 {
            let table = SymTable {
                context,
                ..Default::default()
            };
            self.tables.push(table);
        }
        self.current += 1;
    }

    /// Pop the scope context onto the stack.
    pub fn pop(&mut self) {
        self.current = self.current.saturating_sub(1);
        self.tables.pop();
    }
}
