use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{
        Bounds,
        Expression,
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
            ScopeContext::DeclarationBounds,
        );
        let fields = contract.models[model_delay.i].fields(contract);

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

        contract.models[model_delay.i].scope = scope;
        contract.models[model_delay.i].bounds = Some(Bounds {
            loc: st.loc.clone(),
            exprs: bounds,
        });
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

        let members = state.fields(contract);

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

        let Ok(bounds) = resolve_bound_exprs(&st.expr, &mut scope, contract) else {
            continue;
        };

        contract.states[state_delay.i].bounds = Some(Bounds {
            loc: st.loc.clone(),
            exprs: bounds,
        });
        contract.states[state_delay.i].scope = scope;
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
            contract.functions[func_delay.i].bounds = Some(Bounds {
                loc: st.loc.clone(),
                exprs: bounds,
            });
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
