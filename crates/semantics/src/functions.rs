use folidity_diagnostics::Report;
use folidity_parser::{
    ast as parsed_ast,
    ast::Identifier,
    Span,
};
use indexmap::IndexMap;

use crate::{
    ast::{
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
    global_symbol::{
        GlobalSymbol,
        SymbolInfo,
    },
    types::map_type,
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

    if error {
        return Err(());
    }

    let decl = Function::new(
        func.loc.clone(),
        func.is_init,
        func_vis,
        return_ty,
        func.name.clone(),
        params,
        s_bound,
    );

    // todo: resolve access attributes
    // need to implement expression resolution first
    // to resolve members to concrete types.

    contract.declaration_symbols.insert(
        decl.name.name.clone(),
        GlobalSymbol::Function(SymbolInfo {
            loc: decl.loc.clone(),
            i: function_no,
        }),
    );
    contract.functions.push(decl);

    Ok(function_no)
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
