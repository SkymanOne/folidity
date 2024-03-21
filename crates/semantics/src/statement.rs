use folidity_diagnostics::Report;
use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{
        Assign,
        ForLoop,
        IfElse,
        Iterator,
        Return,
        Statement,
        StatementBlock,
        TypeVariant,
        Variable,
    },
    contract::ContractDefinition,
    expression::expression,
    global_symbol::GlobalSymbol,
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

            let pos = scope.add(
                &var.names[0].clone(),
                ty.clone(),
                expr.clone(),
                VariableKind::Local,
                var.mutable,
                scope.current,
                contract,
            );

            resolved.push(Statement::Variable(Variable {
                loc: stmt.loc().clone(),
                pos,
                names: var.names.clone(),
                mutable: var.mutable,
                ty,
                value: expr,
            }));
            Ok(true)
        }
        parsed_ast::Statement::Assign(a) => {
            let Some((v_i, _)) = scope.find_var_index(&a.name.name) else {
                contract.diagnostics.push(Report::semantic_error(
                    a.name.loc.clone(),
                    String::from("Cannot find the variable"),
                ));
                return Err(());
            };
            // let table = &scope.tables[t_i];
            let mut sym = scope.find_symbol(&v_i).unwrap().clone();

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
            scope.vars.insert(v_i, sym);

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

            scope.push(ScopeContext::Block);

            for b_stmt in &block.statements {
                if !reachable {
                    contract.diagnostics.push(Report::semantic_warning(
                        b_stmt.loc().clone(),
                        String::from("Unreachable statement."),
                    ));
                    return Err(());
                }
                reachable = statement(b_stmt, &mut resolved_parts, scope, contract)?;
            }

            scope.pop();

            resolved.push(Statement::Block(StatementBlock {
                loc: block.loc.clone(),
                statements: resolved_parts,
            }));

            Ok(reachable)
        }
        parsed_ast::Statement::IfElse(branch) => {
            let eval_cond = expression(
                &branch.condition,
                ExpectedType::Concrete(TypeVariant::Bool),
                scope,
                contract,
            )?;

            scope.push(ScopeContext::Block);
            let mut body_stmts = Vec::new();
            let mut reachable_block = statement(
                &parsed_ast::Statement::Block(*branch.body.clone()),
                &mut body_stmts,
                scope,
                contract,
            )?;
            scope.pop();

            let mut other_stmts = Vec::new();
            if let Some(else_block) = &branch.else_part {
                reachable_block |= statement(else_block, &mut other_stmts, scope, contract)?;
            } else {
                reachable_block = true;
            }

            resolved.push(Statement::IfElse(IfElse {
                loc: branch.loc.clone(),
                condition: eval_cond,
                body: body_stmts,
                else_part: other_stmts,
            }));

            Ok(reachable_block)
        }
        parsed_ast::Statement::ForLoop(for_loop) => {
            scope.push(ScopeContext::Loop);

            let mut body = Vec::new();
            let mut reachable;
            statement(
                &parsed_ast::Statement::Variable(for_loop.var.clone()),
                &mut body,
                scope,
                contract,
            )?;
            let eval_cond = expression(
                &for_loop.condition,
                ExpectedType::Concrete(TypeVariant::Bool),
                scope,
                contract,
            )?;
            let eval_incr =
                expression(&for_loop.incrementer, ExpectedType::Empty, scope, contract)?;

            if for_loop.body.statements.is_empty() {
                reachable = true;
            } else {
                reachable = statement(
                    &parsed_ast::Statement::Block(*for_loop.body.clone()),
                    &mut body,
                    scope,
                    contract,
                )?;
            }

            if body.iter().any(|s| matches!(&s, Statement::Skip(_))) {
                reachable = true;
            }

            scope.pop();

            resolved.push(Statement::ForLoop(ForLoop {
                loc: for_loop.loc.clone(),
                var: Box::new(body[0].clone()),
                condition: eval_cond,
                incrementer: eval_incr,
                body,
            }));

            Ok(reachable)
        }
        parsed_ast::Statement::Iterator(it) => {
            scope.push(ScopeContext::Loop);
            let mut body = Vec::new();
            let list_expr = expression(&it.list, ExpectedType::Dynamic(vec![]), scope, contract)?;
            // todo: destructure field in the iterator
            if it.names.len() != 1 {
                contract.diagnostics.push(Report::semantic_error(
                    it.loc.clone(),
                    String::from("Destructor in iterators are currently unsupported."),
                ));
                return Err(());
            }
            for ident in &it.names {
                scope.add(
                    ident,
                    list_expr.ty().clone(),
                    None,
                    VariableKind::Loop,
                    false,
                    scope.current,
                    contract,
                );
            }

            let reachable = statement(
                &parsed_ast::Statement::Block(*it.body.clone()),
                &mut body,
                scope,
                contract,
            )?;

            scope.pop();

            resolved.push(Statement::Iterator(Iterator {
                loc: it.loc.clone(),
                names: it.names.clone(),
                list: list_expr,
                body,
            }));

            Ok(reachable)
        }
        parsed_ast::Statement::Return(ret) => {
            let GlobalSymbol::Function(sym) = &scope.symbol else {
                contract.diagnostics.push(Report::semantic_error(
                    ret.loc.clone(),
                    String::from("Return can only be used inside a function."),
                ));
                return Err(());
            };

            let func = &contract.functions[sym.i];
            let ret_ty = func.return_ty.ty();
            match (ret_ty, &ret.expr) {
                (TypeVariant::Unit, None) => {
                    resolved.push(Statement::Return(Return {
                        loc: ret.loc.clone(),
                        expr: None,
                    }))
                }
                (TypeVariant::Unit, Some(_)) => {
                    contract.diagnostics.push(Report::semantic_error(
                        ret.loc.clone(),
                        String::from("Function does not expect any value to return."),
                    ));
                    return Err(());
                }
                (_, None) => {
                    contract.diagnostics.push(Report::semantic_error(
                        ret.loc.clone(),
                        String::from("Function expects a value to return."),
                    ));
                    return Err(());
                }
                (ty, Some(e)) => {
                    let resolved_expr =
                        expression(e, ExpectedType::Concrete(ty.clone()), scope, contract)?;
                    resolved.push(Statement::Return(Return {
                        loc: ret.loc.clone(),
                        expr: Some(resolved_expr),
                    }))
                }
            }

            Ok(false)
        }
        parsed_ast::Statement::StateTransition(trans) => {
            let GlobalSymbol::Function(sym) = &scope.symbol else {
                contract.diagnostics.push(Report::semantic_error(
                    trans.loc().clone(),
                    String::from("State transition can only happen inside a function."),
                ));
                return Err(());
            };

            let func = &contract.functions[sym.i];
            let Some(bound) = &func.state_bound else {
                contract.diagnostics.push(Report::semantic_error(
                    trans.loc().clone(),
                    String::from("The function does not specify any state transitions."),
                ));
                return Err(());
            };

            let allowed_tys: Vec<TypeVariant> = bound
                .to
                .iter()
                .map(|param| TypeVariant::State(param.ty.clone()))
                .collect();

            let eval_init = expression(
                &trans.clone(),
                ExpectedType::Dynamic(allowed_tys),
                scope,
                contract,
            )?;

            resolved.push(Statement::StateTransition(eval_init));

            Ok(true)
        }
        parsed_ast::Statement::Skip(loc) => {
            let mut i = scope.current;
            while i > 0 {
                if matches!(scope.tables[i].context, ScopeContext::Loop) {
                    resolved.push(Statement::Skip(loc.clone()));
                    return Ok(false);
                }
                i -= 1;
            }

            contract.diagnostics.push(Report::semantic_error(
                loc.clone(),
                String::from("`skip` can only be used inside loops and iterators"),
            ));

            Err(())
        }
        parsed_ast::Statement::Expression(expr) => {
            let resolved_expr = expression(expr, ExpectedType::Empty, scope, contract)?;

            resolved.push(Statement::Expression(resolved_expr));

            Ok(true)
        }
        parsed_ast::Statement::Error(_) => unimplemented!("Error statement can not be evaluated."),
    }
}
