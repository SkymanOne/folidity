use std::collections::HashMap;

use folidity_parser::ast::Identifier;
use indexmap::IndexMap;

use crate::{
    ast::{Expression, TypeVariant},
    contract::ContractDefinition,
};

#[derive(Debug, Clone)]
pub struct VariableSym {
    /// Name of the variable.
    pub name: Identifier,
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
    /// A user defined type
    /// (e.g. Struct, Model, Enum, Function)
    /// which should exist in global namespace.
    User(usize),
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
        name: &Identifier,
        ty: TypeVariant,
        value: Option<Expression>,
        usage: VariableKind,
    ) {
        let current_id = contract.next_var_id;
        contract.next_var_id += 1;

        self.vars.insert(
            current_id,
            VariableSym {
                name: name.clone(),
                ty,
                value,
                usage,
                used: false,
            },
        );

        self.names.insert(name.name.clone(), current_id);
    }

    pub fn ame(&self, pos: usize) -> &str {
        &self.vars[&pos].name.name
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
    pub fn find_var(&self, name: &str) -> Option<usize> {
        if let Some(i) = self.symbols.names.get(name) {
            return Some(*i);
        } else if let Some(scope) = &self.parent {
            return scope.find_var(name);
        } else {
            None
        }
    }
}
