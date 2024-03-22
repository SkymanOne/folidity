use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{
        Expression,
        FuncReturnType,
        StateBody,
        StateParam,
        TypeVariant,
    },
    contract::ContractDefinition,
    expression::expression,
    global_symbol::{
        GlobalSymbol,
        SymbolInfo,
    },
    symtable::{
        Scope,
        ScopeContext,
        VariableKind,
    },
    types::{
        DelayedDeclarations,
        ExpectedType,
    },
};

/// Resolve `st` model bounds on states, models and functions.
pub fn resolve_bounds(contract: &mut ContractDefinition, delay: &DelayedDeclarations) {
    for model_delay in &delay.models {
        let Some(st) = &model_delay.decl.st_block else {
            continue;
        };
        let mut scope = Scope::new(
            &GlobalSymbol::Model(SymbolInfo {
                loc: model_delay.decl.loc.clone(),
                i: model_delay.i,
            }),
            ScopeContext::Bounds,
        );
        if let Some(parent_i) = &contract.models[model_delay.i].parent {
            let parent = contract.models[*parent_i].clone();
            parent.fields.iter().for_each(|p| {
                scope.add(
                    &p.name,
                    p.ty.ty.clone(),
                    None,
                    VariableKind::Local,
                    false,
                    scope.current,
                    contract,
                );
            });
        }

        let Ok(bounds) = resolve_bound_exprs(&st.expr, &mut scope, contract) else {
            continue;
        };

        contract.models[model_delay.i].bounds = bounds;
    }

    for state_delay in &delay.states {
        let Some(st) = &state_delay.decl.st_block else {
            continue;
        };
        let mut scope = Scope::new(
            &GlobalSymbol::State(SymbolInfo {
                loc: state_delay.decl.loc.clone(),
                i: state_delay.i,
            }),
            ScopeContext::Bounds,
        );

        let state = contract.states[state_delay.i].clone();

        if let Some((parent, Some(ident))) = &state.from {
            scope.add(
                ident,
                TypeVariant::State(parent.clone()),
                None,
                VariableKind::State,
                false,
                scope.current,
                contract,
            );
        }

        if let Some(body) = &state.body {
            let members = match body {
                StateBody::Raw(params) => params.clone(),
                StateBody::Model(s) => contract.models[s.i].fields.clone(),
            };

            members.iter().for_each(|p| {
                scope.add(
                    &p.name,
                    p.ty.ty.clone(),
                    None,
                    VariableKind::Local,
                    false,
                    scope.current,
                    contract,
                );
            });
        }

        let Ok(bounds) = resolve_bound_exprs(&st.expr, &mut scope, contract) else {
            continue;
        };

        contract.states[state_delay.i].bounds = bounds;
    }

    for func_delay in &delay.functions {
        let Some(st) = &func_delay.decl.st_block else {
            continue;
        };
        let mut scope = Scope::default();
        std::mem::swap(&mut contract.functions[func_delay.i].scope, &mut scope);

        scope.push(ScopeContext::Bounds);

        let func_state = contract.functions[func_delay.i].state_bound.clone();
        let func_return = contract.functions[func_delay.i].return_ty.clone();

        let mut add_state_param = |param: &StateParam| {
            if let Some(ident) = &param.name {
                scope.add(
                    ident,
                    TypeVariant::State(param.ty.clone()),
                    None,
                    VariableKind::State,
                    false,
                    scope.current,
                    contract,
                );
            }
        };

        if let Some(b) = &func_state {
            if let Some(from) = &b.from {
                add_state_param(from);
            }
            for state_param in &b.to {
                add_state_param(state_param);
            }
        }

        if let FuncReturnType::ParamType(param) = &func_return {
            scope.add(
                &param.name,
                param.ty.ty.clone(),
                None,
                VariableKind::Return,
                false,
                scope.current,
                contract,
            );
        }

        let Ok(bounds) = resolve_bound_exprs(&st.expr, &mut scope, contract) else {
            continue;
        };

        std::mem::swap(&mut scope, &mut contract.functions[func_delay.i].scope);
        contract.functions[func_delay.i].bounds = bounds;
    }
}

fn resolve_bound_exprs(
    expr: &parsed_ast::Expression,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
) -> Result<Vec<Expression>, ()> {
    let mut bounds = Vec::new();
    let Ok(resolved) = expression(
        expr,
        ExpectedType::Dynamic(vec![
            TypeVariant::Bool,
            TypeVariant::List(Box::new(TypeVariant::Bool)),
        ]),
        scope,
        contract,
    ) else {
        return Err(());
    };

    if let Expression::List(list) = &resolved {
        bounds.extend(list.element.iter().cloned());
    } else {
        bounds.push(resolved);
    }
    Ok(bounds)
}
