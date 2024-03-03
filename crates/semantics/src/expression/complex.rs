//! Resolve complex expressions.

use folidity_diagnostics::Report;
use folidity_parser::ast::Identifier;

use crate::{
    ast::{Expression, FunctionType, TypeVariant, UnaryExpression},
    contract::ContractDefinition,
    global_symbol::SymbolKind,
    symtable::{Scope, VariableSym},
    types::{report_type_mismatch, ExpectedType},
};

/// Resolve variable to a AST expression.
pub fn resolve_variable(
    ident: &Identifier,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    match &expected_ty {
        ExpectedType::Concrete(ty) => {
            if matches!(ty, TypeVariant::Function(_)) {
                let symbol = contract
                    .find_global_symbol(ident, SymbolKind::Function)
                    .ok_or(())?;

                let func = &contract.functions[symbol.i];

                let TypeVariant::Function(f_ty) = ty else {
                    return Err(());
                };

                if func.params.len() != f_ty.params.len() {
                    contract.diagnostics.push(Report::semantic_error(
                        ident.loc.clone(),
                        String::from("Function has invalid number of parameters."),
                    ));
                    return Err(());
                }

                let mut error_params = false;
                let mut error_rty = false;

                for (f_p, fty_p) in func.params.values().zip(f_ty.params.iter()) {
                    if &f_p.ty.ty != fty_p {
                        error_params = true;
                    }
                }

                if func.return_ty.ty() != f_ty.returns.as_ref() {
                    error_rty = true;
                }

                if error_params {
                    contract.diagnostics.push(Report::type_error(
                        ident.loc.clone(),
                        String::from("Function's parameter types mismatched the expected ones."),
                    ));
                }

                if error_rty {
                    contract.diagnostics.push(Report::type_error(
                        ident.loc.clone(),
                        String::from("Function's return type mismatched."),
                    ));
                }

                if error_params || error_rty {
                    return Err(());
                }

                Ok(Expression::Variable(UnaryExpression {
                    loc: ident.loc.clone(),
                    element: symbol.i,
                    ty: TypeVariant::Function(FunctionType {
                        params: f_ty.params.clone(),
                        returns: f_ty.returns.clone(),
                    }),
                }))
            } else {
                let var_id = find_var(ident, contract, scope)?;
                let sym = scope.symbols.vars.get(&var_id).unwrap();

                if &sym.ty != ty {
                    report_type_mismatch(
                        &ExpectedType::Concrete(ty.clone()),
                        &sym.ty,
                        &ident.loc,
                        contract,
                    );
                    return Err(());
                }

                Ok(Expression::Variable(UnaryExpression {
                    loc: ident.loc.clone(),
                    element: var_id,
                    ty: sym.ty.clone(),
                }))
            }
        }
        ExpectedType::Dynamic(tys) => {
            let var_id = find_var(ident, contract, scope)?;
            let sym = scope.symbols.vars.get(&var_id).unwrap();

            if !tys.contains(&sym.ty) {
                contract.diagnostics.push(Report::type_error(
                    ident.loc.clone(),
                    String::from("Variable is not of any allowed types."),
                ));
                return Err(());
            }

            Ok(Expression::Variable(UnaryExpression {
                loc: ident.loc.clone(),
                element: var_id,
                ty: sym.ty.clone(),
            }))
        }
        ExpectedType::Empty => {
            contract.diagnostics.push(Report::semantic_error(
                ident.loc.clone(),
                String::from("Variables can only be used in expression."),
            ));
            Err(())
        }
    }
}

fn find_var<'a>(
    ident: &'a Identifier,
    contract: &'a mut ContractDefinition,
    scope: &'a mut Scope,
) -> Result<usize, ()> {
    let Some(v) = scope.find_var(&ident.name) else {
        contract.diagnostics.push(Report::semantic_error(
            ident.loc.clone(),
            String::from("Variable is not declared"),
        ));
        return Err(());
    };
    Ok(v)
}