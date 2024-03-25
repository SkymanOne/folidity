use std::collections::HashMap;

use folidity_diagnostics::Report;
use folidity_parser::{
    ast::{
        self as parsed_ast,
        Identifier,
        Source,
    },
    Span,
};
use indexmap::IndexMap;

use crate::ast::{
    EnumDeclaration,
    Function,
    ModelDeclaration,
    Param,
    StateBody,
    StateDeclaration,
    StructDeclaration,
};

use crate::{
    functions::function_decl,
    global_symbol::{
        GlobalSymbol,
        SymbolInfo,
        SymbolKind,
    },
    types::{
        find_user_type_recursion,
        map_type,
        validate_fields,
        DelayedDeclaration,
        DelayedDeclarations,
    },
};

/// Arbitrary limit of a max number of topic.
/// To be determined later.
const MAX_ENUM_ITEMS: usize = 120;
/// List of reserved type names that shouldn't be used as the name for a declaration.
const RESERVED_TYPE_NAMES: &[&str] = &[
    "model", "state", "enum", "fn", "mapping", "list", "set", "int", "uint", "float", "string",
    "address", "hex", "char", "bool", "unit",
];

/// Semantically analysed contract definition.
/// Ready for the the next stage of compilation.
#[derive(Debug, Clone, Default)]
pub struct ContractDefinition {
    /// List of all enums in the contract.
    pub enums: Vec<EnumDeclaration>,
    /// List of all structs in the contract.
    pub structs: Vec<StructDeclaration>,
    /// List of all models in the contract.
    pub models: Vec<ModelDeclaration>,
    /// List of all states in the contract.
    pub states: Vec<StateDeclaration>,
    /// list of all functions in the contract.
    pub functions: Vec<Function>,
    /// Mapping from identifiers to global declaration symbols.
    pub declaration_symbols: HashMap<String, GlobalSymbol>,
    /// Id of the next variable in the sym table.
    pub next_var_id: usize,
    /// Errors during semantic analysis.
    pub diagnostics: Vec<Report>,
}

impl ContractDefinition {
    /// Resolve user defined structures: enums, models, states.
    pub fn resolve_declarations(&mut self, tree: &Source) -> DelayedDeclarations {
        let mut delay = DelayedDeclarations {
            structs: Vec::new(),
            models: Vec::new(),
            states: Vec::new(),
            functions: Vec::new(),
        };

        for item in &tree.declarations {
            match item {
                parsed_ast::Declaration::EnumDeclaration(enum_) => self.analyze_enum(enum_),
                parsed_ast::Declaration::StructDeclaration(struct_) => {
                    self.analyze_struct(struct_, &mut delay)
                }
                parsed_ast::Declaration::ModelDeclaration(model) => {
                    self.analyze_model(model, &mut delay)
                }
                parsed_ast::Declaration::StateDeclaration(state) => {
                    self.analyze_state(state, &mut delay)
                }
                _ => (),
            }
        }

        delay
    }

    /// Resolve function signatures
    /// and adds it to the global symbol table.
    pub fn resolve_functions(&mut self, tree: &Source, delayed_decls: &mut DelayedDeclarations) {
        for f in tree.declarations.iter().filter_map(|d| {
            match d {
                parsed_ast::Declaration::FunDeclaration(func) => Some(func),
                _ => None,
            }
        }) {
            if let Ok(id) = function_decl(f, self) {
                delayed_decls.functions.push(DelayedDeclaration {
                    i: id,
                    decl: *f.clone(),
                });
            }
        }
    }

    /// Resolves fields during the second pass.
    /// - Discover fields for structs, models, and states.
    /// - Detect any cycles and report them.
    /// - Ensure that no fields have types of any state or model.
    pub fn resolve_fields(&mut self, delay: &DelayedDeclarations) {
        // Update fields of the models and struct.
        for s in &delay.structs {
            let s_fields = self.analyze_fields(&s.decl.fields, &s.decl.name);
            self.structs[s.i].fields = s_fields;
        }

        for m in &delay.models {
            let m_fields = self.analyze_fields(&m.decl.fields, &m.decl.name);
            self.models[m.i].fields = m_fields;
        }

        for state in &delay.states {
            let body = match &state.decl.body {
                Some(parsed_ast::StateBody::Raw(params)) => {
                    let fields = self.analyze_fields(params, &state.decl.name);
                    Some(StateBody::Raw(fields))
                }
                // If the body is a model, then we need to resolve the model symbol in the
                // symbol table
                Some(parsed_ast::StateBody::Model(ident)) => {
                    let Some(symbol) = GlobalSymbol::lookup(self, ident) else {
                        continue;
                    };
                    match symbol {
                        GlobalSymbol::Model(m) => Some(StateBody::Model(m.clone())),
                        // The symbol must be a model, otherwise the type is invalid.
                        _ => {
                            self.diagnostics.push(Report::semantic_error(
                                ident.loc.clone(),
                                String::from("Expected model, found other type."),
                            ));
                            continue;
                        }
                    }
                }
                None => None,
            };

            self.states[state.i].body = body;
        }

        find_user_type_recursion(self);
        validate_fields(self);
    }

    /// Resolve fields of declarations.
    fn analyze_fields(&mut self, fields: &[parsed_ast::Param], ident: &Identifier) -> Vec<Param> {
        let mut analyzed_fields: Vec<Param> = Vec::new();
        if fields.is_empty() {
            self.diagnostics.push(Report::semantic_error(
                ident.loc.clone(),
                format!("`{}` has no fields", &ident.name),
            ));
            return analyzed_fields;
        }

        let mut duplicates = Vec::new();
        for field in fields {
            if analyzed_fields
                .iter()
                .any(|f| f.name.name == field.name.name)
            {
                duplicates.push(field)
            }
            if !duplicates.is_empty() {
                let start = duplicates
                    .iter()
                    .min_by(|x, y| x.loc.start.cmp(&y.loc.start))
                    .map(|p| p.loc.start)
                    .unwrap();
                let end = duplicates
                    .iter()
                    .max_by(|x, y| x.loc.end.cmp(&y.loc.end))
                    .map(|p| p.loc.end)
                    .unwrap();

                self.diagnostics.push(Report::semantic_error(
                    Span { start, end },
                    format!("`{}` is duplicated", field.name.name),
                ));
            }

            let Ok(param_type) = map_type(self, &field.ty) else {
                continue;
            };
            let param = Param {
                loc: field.loc.clone(),
                ty: param_type,
                name: field.name.clone(),
                is_mut: field.is_mut,
                recursive: false,
            };

            analyzed_fields.push(param);
        }
        analyzed_fields
    }

    /// Resolves enum declarations. This is done in one pass.
    fn analyze_enum(&mut self, item: &parsed_ast::EnumDeclaration) {
        if item.variants.is_empty() {
            self.diagnostics.push(Report::semantic_error(
                item.loc.clone(),
                String::from("Enum must have at least one element."),
            ))
        } else if item.variants.len() > MAX_ENUM_ITEMS {
            self.diagnostics.push(Report::semantic_error(
                item.loc.clone(),
                format!("Exceeded the {}-item limit of enum", MAX_ENUM_ITEMS),
            ));
        }

        let mut entries: IndexMap<String, Span> = IndexMap::new();

        for e in item.variants.iter() {
            if entries.get(&e.name).is_some() {
                self.diagnostics.push(Report::semantic_error(
                    e.loc.clone(),
                    format!("`{}` has already been defined", e.name),
                ));
                continue;
            }
            entries.insert(e.name.clone(), e.loc.clone());
        }

        let decl = EnumDeclaration {
            loc: item.loc.clone(),
            name: item.name.clone(),
            variants: entries,
        };

        let pos = self.enums.len();

        self.enums.push(decl);

        self.add_global_symbol(
            &item.name.clone(),
            GlobalSymbol::Enum(SymbolInfo::new(item.loc.clone(), pos)),
        );
    }

    /// Analyses struct declaration creating a delay in the symbol table.
    fn analyze_struct(
        &mut self,
        item: &parsed_ast::StructDeclaration,
        delay: &mut DelayedDeclarations,
    ) {
        let struct_len = self.structs.len();
        // if we successfully add a symbol to the symbol table,
        // then we can proceed with creating the delayed fields for the second pass.
        if self.add_global_symbol(
            &item.name,
            GlobalSymbol::Struct(SymbolInfo::new(item.loc.clone(), struct_len)),
        ) {
            self.structs.push(StructDeclaration {
                loc: item.loc.clone(),
                name: item.name.clone(),
                fields: Vec::new(),
            });

            delay
                .structs
                .push(DelayedDeclaration::<parsed_ast::StructDeclaration> {
                    decl: item.clone(),
                    i: struct_len,
                });
        }
    }

    /// Same as `analyze_struct`
    fn analyze_model(
        &mut self,
        item: &parsed_ast::ModelDeclaration,
        delay: &mut DelayedDeclarations,
    ) {
        let model_len = self.models.len();
        // if we successfully add a symbol to the symbol table,
        // then we can proceed with creating the delayed fields for the second pass.
        if self.add_global_symbol(
            &item.name,
            GlobalSymbol::Model(SymbolInfo::new(item.loc.clone(), model_len)),
        ) {
            self.models.push(ModelDeclaration {
                loc: item.loc.clone(),
                name: item.name.clone(),
                fields: Vec::new(),
                parent: None,
                bounds: Vec::new(),
                recursive_parent: false,
            });

            delay
                .models
                .push(DelayedDeclaration::<parsed_ast::ModelDeclaration> {
                    decl: item.clone(),
                    i: model_len,
                });
        }
    }

    fn analyze_state(
        &mut self,
        item: &parsed_ast::StateDeclaration,
        delay: &mut DelayedDeclarations,
    ) {
        let state_len = self.states.len();
        // if we successfully add a symbol to the symbol table,
        // then we can proceed with creating the delayed fields for the second pass.
        if self.add_global_symbol(
            &item.name,
            GlobalSymbol::State(SymbolInfo::new(item.loc.clone(), state_len)),
        ) {
            self.states.push(StateDeclaration {
                loc: item.loc.clone(),
                name: item.name.clone(),
                body: None,
                from: None,
                bounds: Vec::new(),
                recursive_parent: false,
            });

            delay
                .states
                .push(DelayedDeclaration::<parsed_ast::StateDeclaration> {
                    decl: item.clone(),
                    i: state_len,
                });
        }
    }

    /// Add a symbol to the global symbol table.
    ///
    /// # Errors
    /// - The symbol table is already in use.
    /// - The symbol name is a reserved word.
    pub fn add_global_symbol(&mut self, ident: &Identifier, symbol: GlobalSymbol) -> bool {
        if RESERVED_TYPE_NAMES.contains(&ident.name.as_str()) {
            self.diagnostics.push(Report::semantic_error(
                ident.loc.clone(),
                String::from("The symbol uses the reserved typename."),
            ));
            return false;
        }

        if let Some(s) = self.declaration_symbols.get(&ident.name) {
            let error_type = match s {
                GlobalSymbol::Struct(_) => "struct",
                GlobalSymbol::Model(_) => "model",
                GlobalSymbol::Enum(_) => "enum",
                GlobalSymbol::State(_) => "state",
                GlobalSymbol::Function(_) => "function",
            };
            let err_msg = format!(
                "The {} `{}` has already been defined earlier.",
                error_type, ident.name
            );

            self.diagnostics
                .push(Report::semantic_error(ident.loc.clone(), err_msg));
            return false;
        }

        self.declaration_symbols.insert(ident.name.clone(), symbol);

        true
    }

    /// Find symbol in a global symbol table.
    ///
    /// # Notes
    /// - Returns `None` if no symbol with provided name has been declared.
    /// - Returns `None` if the found symbol is of different type.
    pub fn find_global_symbol(
        &mut self,
        ident: &Identifier,
        kind: SymbolKind,
    ) -> Option<SymbolInfo> {
        let report_error = |contract: &mut ContractDefinition, expected: String, found: String| {
            contract.diagnostics.push(Report::type_error(
                ident.loc.clone(),
                format!("Expected to find {}, found {}", expected, found),
            ));
        };
        let Some(sym) = GlobalSymbol::lookup(self, ident) else {
            return None;
        };
        match &kind {
            SymbolKind::Struct => {
                if let GlobalSymbol::Struct(s) = sym {
                    Some(s.clone())
                } else {
                    report_error(self, SymbolKind::Struct.to_string(), kind.to_string());
                    None
                }
            }
            SymbolKind::Model => {
                if let GlobalSymbol::Model(s) = sym {
                    Some(s.clone())
                } else {
                    report_error(self, SymbolKind::Model.to_string(), kind.to_string());
                    None
                }
            }
            SymbolKind::State => {
                if let GlobalSymbol::State(s) = sym {
                    Some(s.clone())
                } else {
                    report_error(self, SymbolKind::State.to_string(), kind.to_string());
                    None
                }
            }
            SymbolKind::Enum => {
                if let GlobalSymbol::Enum(s) = sym {
                    Some(s.clone())
                } else {
                    report_error(self, SymbolKind::Enum.to_string(), kind.to_string());
                    None
                }
            }
            SymbolKind::Function => {
                if let GlobalSymbol::Function(s) = sym {
                    Some(s.clone())
                } else {
                    report_error(self, SymbolKind::Function.to_string(), kind.to_string());
                    None
                }
            }
        }
    }
}
