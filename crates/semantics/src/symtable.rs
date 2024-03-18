use std::collections::HashMap;

use folidity_parser::ast::Identifier;
use indexmap::IndexMap;

use crate::{
    ast::{
        Expression,
        TypeVariant,
    },
    contract::ContractDefinition,
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
    // /// A user defined type
    // /// (e.g. Struct, Model, Enum, Function)
    // /// which should exist in global namespace.
    // User(usize),
}

/// Context of the scope in the symtable.
#[derive(Debug, Clone, Default)]
pub enum ScopeContext {
    Bounds,
    Function,
    #[default]
    Global,
    Loop,
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
            },
        );

        self.names.insert(ident.name.clone(), current_id);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    /// Parent scope.
    pub parent: Option<Box<Scope>>,
    /// Symbols within the current scope.
    pub symbols: SymTable,
    /// Child scope.
    pub child: Option<Box<Scope>>,
}

impl Scope {
    /// Attempts to find an index of a symbol in the current or outer scopes.
    ///
    /// # Returns
    /// - Index of a symbol in the table if found.
    /// - Reference to the table where the symbol can be found if any.
    pub fn find_var_index(&self, name: &str) -> Option<(usize, &SymTable)> {
        if let Some(i) = self.symbols.names.get(name) {
            Some((*i, &self.symbols))
        } else if let Some(scope) = &self.parent {
            scope.find_var_index(name)
        } else {
            None
        }
    }

    /// Attempts to find a symbol with the given symbol in the current or outer scopes.
    ///
    /// # Returns
    /// - A reference to the symbol in the table if any
    pub fn find_symbol(&self, index: &usize) -> Option<&VariableSym> {
        if let Some(s) = self.symbols.vars.get(index) {
            Some(s)
        } else if let Some(scope) = &self.parent {
            scope.find_symbol(index)
        } else {
            None
        }
    }
}
