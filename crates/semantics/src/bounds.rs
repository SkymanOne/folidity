use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{
        Expression,
        StateBody,
        TypeVariant,
    },
    contract::ContractDefinition,
    expression::{
        expression,
        resolve_nested_fields,
    },
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
            ScopeContext::DeclarationBounds,
        );
        let mut fields = Vec::new();
        let parent_sym = contract.models[model_delay.i].parent.clone();
        resolve_nested_fields(&parent_sym, &mut fields, contract);
        let model_fields = contract.models[model_delay.i].fields.clone();

        fields.extend(model_fields);

        for f in fields {
            scope.add(
                &f.name,
                f.ty.ty.clone(),
                None,
                VariableKind::Local,
                false,
                scope.current,
                contract,
            );
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
            ScopeContext::DeclarationBounds,
        );

        let state = contract.states[state_delay.i].clone();

        if let Some((parent, Some(ident))) = &state.from {
            scope.add(
                ident,
                TypeVariant::State(parent.clone()),
                None,
                VariableKind::FromState,
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
        let mut scope = Scope::default();
        std::mem::swap(&mut contract.functions[func_delay.i].scope, &mut scope);

        if let Some(st) = &func_delay.decl.st_block {
            let bounds = if let Ok(exprs) = resolve_bound_exprs(&st.expr, &mut scope, contract) {
                exprs
            } else {
                vec![]
            };
            contract.functions[func_delay.i].bounds = bounds;
        }

        std::mem::swap(&mut scope, &mut contract.functions[func_delay.i].scope);
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
