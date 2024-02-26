use folidity_diagnostics::Report;
use folidity_parser::ast as parsed_ast;
use folidity_parser::Span;
use indexmap::IndexMap;

use crate::ast::FuncReturnType;
use crate::ast::Param;
use crate::ast::TypeVariant;
use crate::contract::ContractDefinition;
use crate::types::map_type;

/// Parses the function declaration without the body.
pub fn function_decl(
    func: &parsed_ast::FunctionDeclaration,
    contract: &mut ContractDefinition,
) -> Result<usize, ()> {
    let error = false;
    let function_no = contract.functions.len();

    let params = resolve_func_param(&func.params, contract)?;
    let return_ty = resolve_func_return(
        &func.return_ty,
        params
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>()
            .as_slice(),
        contract,
    );
    //todo: check annotations

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
