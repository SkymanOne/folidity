use folidity_diagnostics::Report;
use folidity_parser::{
    ast as parsed_ast,
    ast::Identifier,
    Span,
};
use indexmap::IndexMap;

use crate::{
    ast::{
        Expression,
        FuncReturnType,
        Function,
        FunctionVisibility,
        Param,
        StateBound,
        StateParam,
        Type,
        TypeVariant,
        ViewState,
    },
    contract::ContractDefinition,
    expression::expression,
    global_symbol::{
        GlobalSymbol,
        SymbolInfo,
    },
    statement::statement,
    symtable::{
        Scope,
        ScopeContext,
        VariableKind,
    },
    types::{
        map_type,
        ExpectedType,
    },
};

/// Parses the function declaration without the body.
pub fn function_decl(
    func: &parsed_ast::FunctionDeclaration,
    contract: &mut ContractDefinition,
) -> Result<usize, ()> {
    let mut error = false;
    let function_no = contract.functions.len();

    let params = match resolve_func_param(&func.params, contract) {
        Ok(v) => v,
        Err(()) => {
            error = true;
            IndexMap::default()
        }
    };
    let return_ty = match resolve_func_return(
        &func.return_ty,
        params
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>()
            .as_slice(),
        contract,
    ) {
        Ok(v) => v,
        Err(()) => {
            error = true;
            FuncReturnType::Type(Type::new(0, 0, TypeVariant::Int))
        }
    };

    // Check if the function attributes do not conflict with each other.
    if func.is_init {
        match &func.vis {
            &parsed_ast::FunctionVisibility::Priv => {
                contract.diagnostics.push(Report::semantic_error(
                    func.loc.clone(),
                    String::from("Initialising functions cannot be private."),
                ));
                error = true;
            }
            parsed_ast::FunctionVisibility::View(v) => {
                contract.diagnostics.push(Report::semantic_error(
                    v.loc.clone(),
                    String::from("Initialising functions cannot be views."),
                ));
                error = true;
            }
            _ => {}
        }
    }

    let mut func_vis = FunctionVisibility::Priv;
    if let parsed_ast::FunctionVisibility::View(v) = &func.vis {
        let mut view_error = false;
        let mut id = 0;
        let mut ident = Identifier::default();
        if func.access_attributes.is_empty() {
            contract.diagnostics.push(Report::semantic_warning(
                v.loc.clone(),
                String::from(
                    "This view function is inaccessible and will be omitted from the final build.",
                ),
            ));
        }

        if let Some(sym) = GlobalSymbol::lookup(contract, &v.param.ty) {
            if let GlobalSymbol::State(s) = sym {
                id = s.i;
            } else {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    String::from("You can only view states."),
                ));
                view_error = true;
            }
        } else {
            view_error = true;
        }

        if let Some(name) = &v.param.name {
            if contract.declaration_symbols.get(&name.name).is_some() {
                contract.diagnostics.push(Report::semantic_error(
                    name.loc.clone(),
                    String::from("This identifier has been declared before."),
                ));
                view_error = true;
            } else {
                ident = name.clone();
            }
        } else {
            contract.diagnostics.push(Report::semantic_error(
                v.loc.clone(),
                String::from("State binding variable must be specified."),
            ));
            view_error = true;
        }

        if view_error {
            error = true;
        } else {
            func_vis = FunctionVisibility::View(ViewState {
                loc: v.loc.clone(),
                ty: id,
                name: ident,
            })
        }
    } else {
        match &func.vis {
            parsed_ast::FunctionVisibility::Pub => {
                func_vis = FunctionVisibility::Pub;
            }
            parsed_ast::FunctionVisibility::Priv => {
                func_vis = FunctionVisibility::Priv;
            }
            _ => {}
        }
    }
    let s_bound = if let Some(state_bound) = &func.state_bound {
        match resolve_func_state_bound(state_bound, contract) {
            Ok(v) => Some(v),
            Err(_) => {
                error = true;
                None
            }
        }
    } else {
        None
    };

    let sym = GlobalSymbol::Function(SymbolInfo {
        loc: func.loc.clone(),
        i: function_no,
    });

    let mut scope = Scope::new(&sym, ScopeContext::AccessAttributes);
    scope.add(
        &Identifier {
            loc: func.loc.clone(),
            name: "any".to_string(),
        },
        TypeVariant::Address,
        None,
        VariableKind::State,
        false,
        scope.current,
        contract,
    );
    if let Some(bounds) = &s_bound {
        if let Some(from) = &bounds.from {
            if let Some(var) = &from.name {
                scope.add(
                    var,
                    TypeVariant::State(from.ty.clone()),
                    None,
                    VariableKind::State,
                    false,
                    scope.current,
                    contract,
                );
            }
        }
    }

    let access_attributes: Vec<Expression> = func
        .access_attributes
        .iter()
        .flat_map(|attr| &attr.members)
        .filter_map(|expr| {
            match expression(
                expr,
                ExpectedType::Dynamic(vec![
                    TypeVariant::Address,
                    TypeVariant::Set(Box::new(TypeVariant::Address)),
                    TypeVariant::List(Box::new(TypeVariant::Address)),
                ]),
                &mut scope,
                contract,
            ) {
                Ok(v) => Some(v),
                Err(_) => {
                    error = true;
                    None
                }
            }
        })
        .collect();

    if let Some(any) = access_attributes
        .iter()
        .find(|x| x.is_access_wildcard(&scope))
    {
        if access_attributes.len() > 1 {
            contract.diagnostics.push(Report::semantic_error(
                any.loc().clone(),
                String::from("Wildcard `any` cannot be used with other members."),
            ));

            error = true;
        }
    }

    if error {
        return Err(());
    }

    let mut decl = Function::new(
        func.loc.clone(),
        func.is_init,
        func_vis,
        return_ty,
        func.name.clone(),
        params,
        s_bound,
    );

    decl.scope = scope;
    decl.access_attributes = access_attributes;

    contract
        .declaration_symbols
        .insert(decl.name.name.clone(), sym);
    contract.functions.push(decl);

    Ok(function_no)
}

/// Resolve function body.
/// - Creates a scope and add parameters there.
/// - Traverses statement tree and adds resolved statements to the body list.
/// - Check for reachability
/// # Errors
/// - No `return` is provided if expected.
/// - Errors during parsing of statements.
pub fn resolve_func_body(
    func_decl: &parsed_ast::FunctionDeclaration,
    func_i: usize,
    contract: &mut ContractDefinition,
) -> Result<(), ()> {
    let mut scope = Scope::default();
    std::mem::swap(&mut scope, &mut contract.functions[func_i].scope);
    scope.push(ScopeContext::FunctionParams);

    let mut resolved_stmts = Vec::new();

    // add params to the scope.
    for param in &func_decl.params {
        let ty = map_type(contract, &param.ty)?;
        scope.add(
            &param.name,
            ty.ty,
            None,
            VariableKind::Param,
            param.is_mut,
            scope.current,
            contract,
        );
    }

    scope.push(ScopeContext::FunctionBody);

    // if the return type is not `()` then we expect the function body to contain `return`
    // statement. i.e. it should be unreachable after the last statement,
    let return_required = !matches!(func_decl.return_ty.ty(), parsed_ast::TypeVariant::Unit);
    let mut transition_required = false;
    if let Some(state_bound) = &func_decl.state_bound {
        if !state_bound.to.is_empty() {
            transition_required = true;
        }
    }
    let mut mutating = false;
    let reachable = statement(
        &func_decl.body,
        &mut resolved_stmts,
        &mut scope,
        &mut mutating,
        contract,
    )?;

    if reachable && return_required {
        contract.diagnostics.push(Report::semantic_error(
            func_decl.loc.clone(),
            format!(
                "Expected function to return a value of type {}",
                contract.functions[func_i].return_ty.ty().display(contract)
            ),
        ));
    }

    if !mutating && transition_required {
        let bounds = func_decl.state_bound.as_ref().unwrap();
        let states: String = bounds
            .to
            .iter()
            .fold(String::new(), |init, s| format!("{} {}", &init, &s.ty.name))
            .trim()
            .to_string();
        contract.diagnostics.push(Report::semantic_error(
            bounds.loc.clone(),
            format!("Expected function to transition to states [{}]", states),
        ));
    }

    if mutating && !transition_required {
        contract.diagnostics.push(Report::semantic_error(
            func_decl.loc.clone(),
            String::from("Function is not supposed to perform a state transition."),
        ));
    }

    // pop function body scope
    scope.pop();

    contract.functions[func_i].body = resolved_stmts;
    std::mem::swap(&mut scope, &mut contract.functions[func_i].scope);

    Ok(())
}

/// Resolve function parameters.
fn resolve_func_param(
    parsed_params: &[parsed_ast::Param],
    contract: &mut ContractDefinition,
) -> Result<IndexMap<String, Param>, ()> {
    let mut params: IndexMap<String, Param> = IndexMap::new();
    for p in parsed_params {
        if params.get(&p.name.name).is_some() {
            contract.diagnostics.push(Report::semantic_error(
                p.loc.clone(),
                String::from("Parameter with this name exist."),
            ));
        }

        let Ok(ty) = map_type(contract, &p.ty) else {
            continue;
        };

        if !validate_type(&ty.ty, contract, &ty.loc) {
            continue;
        }

        params.insert(
            p.name.name.clone(),
            Param {
                loc: p.loc.clone(),
                ty,
                name: p.name.clone(),
                is_mut: p.is_mut,
                recursive: false,
            },
        );
    }

    Ok(params)
}

fn resolve_func_return(
    parsed_ty: &parsed_ast::FuncReturnType,
    params: &[String],
    contract: &mut ContractDefinition,
) -> Result<FuncReturnType, ()> {
    match parsed_ty {
        parsed_ast::FuncReturnType::Type(ty) => {
            let mapped_ty = map_type(contract, ty)?;
            if !validate_type(&mapped_ty.ty, contract, &ty.loc) {
                return Err(());
            }
            Ok(FuncReturnType::Type(mapped_ty))
        }
        parsed_ast::FuncReturnType::ParamType(pty) => {
            if params.contains(&pty.name.name) {
                contract.diagnostics.push(Report::semantic_error(
                    pty.name.loc.clone(),
                    String::from("The identifier has already been declared in function params"),
                ));
                return Err(());
            }
            let mapped_ty = map_type(contract, &pty.ty)?;
            if !validate_type(&mapped_ty.ty, contract, &mapped_ty.loc) {
                return Err(());
            }
            Ok(FuncReturnType::ParamType(Param {
                loc: pty.loc.clone(),
                ty: mapped_ty,
                name: pty.name.clone(),
                is_mut: false,
                recursive: false,
            }))
        }
    }
}

fn validate_type(ty: &TypeVariant, contract: &mut ContractDefinition, loc: &Span) -> bool {
    match ty {
        TypeVariant::Function(_) => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Function is not a supported parameter type."),
            ));
            false
        }
        TypeVariant::Model(_) => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Model is not a supported parameter type."),
            ));
            false
        }
        TypeVariant::State(_) => {
            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("Model is not a supported parameter type."),
            ));
            false
        }
        _ => true,
    }
}

fn resolve_func_state_bound(
    state_bound: &parsed_ast::StateBound,
    contract: &mut ContractDefinition,
) -> Result<StateBound, ()> {
    let mut lookup_param = |param: &parsed_ast::StateParam| -> Result<StateParam, ()> {
        if let Some(GlobalSymbol::State(sym)) = GlobalSymbol::lookup(contract, &param.ty) {
            Ok(StateParam {
                loc: param.loc.clone(),
                ty: sym.clone(),
                name: param.name.clone(),
            })
        } else {
            contract.diagnostics.push(Report::semantic_error(
                param.ty.loc.clone(),
                String::from("Must be a state."),
            ));
            Err(())
        }
    };
    let from_b = if let Some(from) = &state_bound.from {
        Some(lookup_param(from)?)
    } else {
        None
    };

    let mut error = false;
    let tos: Vec<StateParam> = state_bound
        .to
        .iter()
        .filter_map(|param| {
            match lookup_param(param) {
                Ok(v) => Some(v),
                Err(_) => {
                    error = true;
                    None
                }
            }
        })
        .collect();

    if error {
        return Err(());
    }

    Ok(StateBound {
        loc: state_bound.loc.clone(),
        from: from_b,
        to: tos,
    })
}
