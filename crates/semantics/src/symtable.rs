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
#[derive(Debug, Clone)]
pub enum VariableKind {
    Return,
    Destructor,
    Param,
    Local,
    State,
    Loop,
    // /// A user defined type
    // /// (e.g. Struct, Model, Enum, Function)
    // /// which should exist in global namespace.
    // User(usize),
}

/// Context of the scope in the symtable.
#[derive(Debug, Clone, Default)]
pub enum ScopeContext {
    /// We are inside bound context of some global symbol.
    Bounds,
    /// Scope is in the function with the given index.
    Function,
    #[default]
    Global,
    Loop,
    Block,
}

#[derive(Debug, Clone, Default)]
pub struct SymTable {
    /// Indexed map of variables
    pub vars: IndexMap<usize, VariableSym>,
    /// Variable names in the current scope.
    pub names: HashMap<String, usize>,
    // Context of variables in the given scope.
    pub context: ScopeContext,
}

impl SymTable {
    pub fn add(
        &mut self,
        contract: &mut ContractDefinition,
        ident: &Identifier,
        ty: TypeVariant,
        value: Option<Expression>,
        usage: VariableKind,
        mutable: bool,
    ) {
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

        self.names.insert(ident.name.clone(), current_id);
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
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
            tables: vec![SymTable::default()],
            current: 0,
            symbol: Default::default(),
        }
    }
}

impl Scope {
    /// Attempts to find an index of a symbol in the current or outer scopes.
    ///
    /// # Returns
    /// - Index of a symbol in the table if found.
    /// - Index of the table where the symbol can be found if any.
    pub fn find_var_index(&mut self, name: &str) -> Option<(usize, usize)> {
        let mut table_i = self.current;
        let mut table = &self.tables[table_i];
        let mut v_i = table.names.get(name);
        while table_i > 0 {
            table_i -= 1;
            table = &self.tables[table_i];
            v_i = table.names.get(name);
            if v_i.is_some() {
                break;
            }
        }
        v_i.map(|i| (*i, table_i))
    }

    /// Attempts to find a symbol with the given index in the current or outer scopes.
    ///
    /// # Returns
    /// - A reference to the symbol in the table if any
    pub fn find_symbol(&self, index: &usize) -> Option<&VariableSym> {
        let mut table_i = self.current;
        let mut table = &self.tables[table_i];
        let mut sym = table.vars.get(index);
        while table_i > 0 {
            table_i -= 1;
            table = &self.tables[table_i];
            sym = table.vars.get(index);
            if sym.is_some() {
                break;
            }
        }
        sym
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
