use folidity_diagnostics::Report;
use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{
        Statement,
        Variable,
    },
    contract::ContractDefinition,
    expression::expression,
    symtable::{
        Scope,
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

            scope.symbols.add(
                contract,
                &var.names[0].clone(),
                ty.clone(),
                expr.clone(),
                VariableKind::Local,
            );

            resolved.push(Statement::Variable(Variable {
                loc: var.loc.clone(),
                names: var.names.clone(),
                mutable: var.mutable,
                ty,
                value: expr,
            }));
            Ok(true)
        }
        parsed_ast::Statement::Assign(_) => todo!(),
        parsed_ast::Statement::IfElse(_) => todo!(),
        parsed_ast::Statement::ForLoop(_) => todo!(),
        parsed_ast::Statement::Iterator(_) => todo!(),
        parsed_ast::Statement::Return(_) => todo!(),
        parsed_ast::Statement::Expression(_) => todo!(),
        parsed_ast::Statement::StateTransition(_) => todo!(),
        parsed_ast::Statement::Skip(_) => todo!(),
        parsed_ast::Statement::Block(_) => todo!(),
        parsed_ast::Statement::Error(_) => todo!(),
    }
}
