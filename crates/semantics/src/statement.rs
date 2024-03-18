use folidity_diagnostics::Report;
use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{
        Assign,
        Statement,
        StatementBlock,
        Variable,
    },
    contract::ContractDefinition,
    expression::expression,
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

/// Resolve parsed statement to an evaluated one.
pub fn statement(
    stmt: &parsed_ast::Statement,
    resolved: &mut Vec<Statement>,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
) -> Result<bool, ()> {
    match stmt {
        parsed_ast::Statement::Variable(var) => {
            let (expr, ty) = match (&var.value, &var.ty) {
                (Some(e), Some(ty)) => {
                    let tv = map_type(contract, ty)?.ty;
                    (
                        Some(expression(
                            e,
                            ExpectedType::Concrete(tv.clone()),
                            scope,
                            contract,
                        )?),
                        tv,
                    )
                }
                (Some(e), None) => {
                    let resolved = expression(e, ExpectedType::Dynamic(vec![]), scope, contract)?;
                    let ty = resolved.ty().clone();
                    (Some(resolved), ty)
                }
                (None, Some(ty)) => {
                    let tv = map_type(contract, ty)?;
                    (None, tv.ty)
                }
                _ => {
                    contract.diagnostics.push(Report::type_error(stmt.loc().clone(), String::from("Type can not be inferred. Try annotating the variable or providing and expression.")));
                    return Err(());
                }
            };

            // todo: destructure fields.
            if var.names.len() != 1 {
                contract.diagnostics.push(Report::semantic_error(
                    stmt.loc().clone(),
                    String::from("Destructuring is currently unsupported."),
                ));
                return Err(());
            }

            scope.tables[scope.current].add(
                contract,
                &var.names[0].clone(),
                ty.clone(),
                expr.clone(),
                VariableKind::Local,
                var.mutable,
            );

            resolved.push(Statement::Variable(Variable {
                loc: stmt.loc().clone(),
                names: var.names.clone(),
                mutable: var.mutable,
                ty,
                value: expr,
            }));
            Ok(true)
        }
        parsed_ast::Statement::Assign(a) => {
            let Some((v_i, t_i)) = scope.find_var_index(&a.name.name) else {
                contract.diagnostics.push(Report::semantic_error(
                    a.name.loc.clone(),
                    String::from("Cannot find the variable"),
                ));
                return Err(());
            };
            let table = &scope.tables[t_i];
            let mut sym = table.vars.get(&v_i).unwrap().clone();

            if !sym.mutable {
                contract.diagnostics.push(Report::semantic_error(
                    a.name.loc.clone(),
                    String::from(
                        "Variable is immutable. Annotate with `mut` keyword to allow mutation.",
                    ),
                ));
                return Err(());
            }

            let resolved_value = expression(
                &a.value,
                ExpectedType::Concrete(sym.ty.clone()),
                scope,
                contract,
            )?;

            sym.value = Some(resolved_value.clone());
            scope.tables[t_i].vars.insert(v_i, sym);

            resolved.push(Statement::Assign(Assign {
                loc: a.loc.clone(),
                name: a.name.clone(),
                value: resolved_value,
            }));
            Ok(true)
        }
        parsed_ast::Statement::Block(block) => {
            let mut reachable = true;

            let mut resolved_parts = Vec::new();

            scope.push_scope(ScopeContext::Block);

            for b_stmt in &block.statements {
                if !reachable {
                    contract.diagnostics.push(Report::semantic_error(
                        b_stmt.loc().clone(),
                        String::from("Unreachable statement."),
                    ));
                    return Err(());
                }
                reachable = statement(b_stmt, &mut resolved_parts, scope, contract)?;
            }

            scope.pop_scope();

            resolved.push(Statement::Block(StatementBlock {
                loc: block.loc.clone(),
                statements: resolved_parts,
            }));

            Ok(true)
        }
        parsed_ast::Statement::IfElse(_) => todo!(),
        parsed_ast::Statement::ForLoop(_) => todo!(),
        parsed_ast::Statement::Iterator(_) => todo!(),
        parsed_ast::Statement::Return(_) => todo!(),
        parsed_ast::Statement::Expression(_) => todo!(),
        parsed_ast::Statement::StateTransition(_) => todo!(),
        parsed_ast::Statement::Skip(_) => todo!(),
        parsed_ast::Statement::Error(_) => unimplemented!("Error statement can not be evaluated."),
    }
}
