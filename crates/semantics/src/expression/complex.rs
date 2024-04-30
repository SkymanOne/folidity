//! Resolve complex expressions.

use folidity_diagnostics::{
    Paint,
    Report,
};
use folidity_parser::{
    ast::{
        self as parsed_ast,
        Identifier,
    },
    Span,
};

use crate::{
    ast::{
        self,
        Expression,
        FunctionCall,
        FunctionType,
        MemberAccess,
        Param,
        StateBody,
        StructInit,
        TypeVariant,
        UnaryExpression,
    },
    contract::ContractDefinition,
    global_symbol::{
        GlobalSymbol,
        SymbolInfo,
        SymbolKind,
    },
    symtable::Scope,
    types::{
        report_type_mismatch,
        ExpectedType,
    },
};

use super::expression;

/// Resolve variable to a AST expression.
///
/// # Notes
/// - If the expected type is a function, then we look it up and the scope table
/// and validate that params and return types are matched.
/// - Otherwise we just look up tha variable in the scope table.
/// - If the expected type is dynamic, we check that the var's type is in the list.
///
/// # Errors
/// - If the var is a function, param or return types mismatched with the expected ones.
/// - The var is not declared.
/// - The var type mismatched.
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
            } else if let Some((var_id, _)) = scope.find_var_index(&ident.name) {
                let sym = scope.find_symbol(&var_id).unwrap();
                if &sym.ty != ty {
                    report_type_mismatch(&expected_ty, &[sym.ty.clone()], &ident.loc, contract);
                    return Err(());
                }

                Ok(Expression::Variable(UnaryExpression {
                    loc: ident.loc.clone(),
                    element: var_id,
                    ty: sym.ty.clone(),
                }))
            } else if let Some(sym) = &contract.find_global_symbol(ident, SymbolKind::Enum) {
                // todo: rewrite this to reduce code duplication.
                let enum_ty = TypeVariant::Enum(sym.clone());
                if &enum_ty != ty {
                    report_type_mismatch(&expected_ty, &[enum_ty], &ident.loc, contract);
                    return Err(());
                } else {
                    Ok(Expression::Variable(UnaryExpression {
                        loc: ident.loc.clone(),
                        element: sym.i,
                        ty: enum_ty,
                    }))
                }
            } else {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    format!(
                        "`{}`: Variable is not declared or inaccessible.",
                        ident.name.yellow().bold()
                    ),
                ));
                Err(())
            }
        }
        ExpectedType::Dynamic(tys) => {
            if let Some((var_id, _)) = scope.find_var_index(&ident.name) {
                let sym = scope.find_symbol(&var_id).unwrap();
                if !tys.is_empty() && !tys.contains(&sym.ty) {
                    report_type_mismatch(&expected_ty, &[sym.ty.clone()], &ident.loc, contract);
                    return Err(());
                }

                Ok(Expression::Variable(UnaryExpression {
                    loc: ident.loc.clone(),
                    element: var_id,
                    ty: sym.ty.clone(),
                }))
            } else if let Some(sym) = &contract.find_global_symbol(ident, SymbolKind::Enum) {
                let ty = TypeVariant::Enum(sym.clone());
                if !tys.is_empty() && !tys.contains(&ty) {
                    report_type_mismatch(&expected_ty, &[ty], &ident.loc, contract);
                    return Err(());
                } else {
                    Ok(Expression::Variable(UnaryExpression {
                        loc: ident.loc.clone(),
                        element: sym.i,
                        ty,
                    }))
                }
            } else {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    format!(
                        "`{}`: Variable is not declared or inaccessible.",
                        ident.name.yellow().bold()
                    ),
                ));
                Err(())
            }
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

/// Resolves function call to a concrete ASP expression.
///
/// # Notes
/// - We first resolve the the arguments iteratively and check for any errors.
/// - We then pattern match the function return type with the expected one.
///     - If it is a concrete one, then we simple check for equality.
///     - If it is a dynamic one, then we ensure check that the generic allowed types
///       intersect or contained.
///     - If it is empty, then the return types is ignored and we use whatever it is.
///
/// # Errors
/// - Function is not defined
/// - Number of arguments mismatch
/// - Argument types mismatch
/// - Return types mismatch
pub fn resolve_func_call(
    ident: &Identifier,
    args: &[parsed_ast::Expression],
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    let symbol = contract
        .find_global_symbol(ident, SymbolKind::Function)
        .ok_or(())?;

    let func = &contract.functions[symbol.i].clone();
    if func.params.len() != args.len() {
        report_mismatched_args_len(&loc, func.params.len(), args.len(), contract);
        return Err(());
    }

    let (parsed_args, error_args) = parse_args(
        args,
        func.params
            .iter()
            .map(|p| p.1.clone())
            .collect::<Vec<Param>>()
            .as_slice(),
        scope,
        contract,
    );

    if error_args {
        contract.diagnostics.push(Report::semantic_error(
            loc.clone(),
            String::from("Functional call has invalid arguments."),
        ));
    }
    let return_ty = match &expected_ty {
        ExpectedType::Concrete(ty) => {
            let mut error_return_ty = false;

            if !check_func_return_type(ty, func.return_ty.ty()) {
                contract.diagnostics.push(Report::type_error(
                    loc.clone(),
                    String::from("Functional's return type mismatched the expected one."),
                ));
                error_return_ty = true;
            }

            if error_args || error_return_ty {
                return Err(());
            }

            ty.clone()
        }
        ExpectedType::Dynamic(tys) => {
            if tys.is_empty() {
                func.return_ty.ty().clone()
            } else {
                match func.return_ty.ty() {
                    // if the function type is generic, then we check that there is intersection of
                    // generic types, and we return generic types with the intersection
                    // of allowed types.
                    TypeVariant::Generic(allowed_tys) => {
                        let filtered_tys: Vec<TypeVariant> = allowed_tys
                            .iter()
                            .filter_map(|t| {
                                if tys.contains(t) {
                                    Some(t.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if filtered_tys.is_empty() {
                            contract.diagnostics.push(Report::type_error(
                                loc.clone(),
                                String::from("Functional's return type cannot be derived."),
                            ));
                            return Err(());
                        }
                        TypeVariant::Generic(filtered_tys)
                    }
                    // same as for generic, but encapsulated inside list type.
                    // If the list type is concrete, then we return the concrete type.
                    TypeVariant::List(l_ty) => {
                        let list_tys: Vec<TypeVariant> = tys
                            .iter()
                            .filter_map(|t| {
                                if let TypeVariant::List(l) = t {
                                    Some(l.as_ref().clone())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        match l_ty.as_ref() {
                            TypeVariant::Generic(g_tys) => {
                                let filtered_tys = list_tys
                                    .iter()
                                    .filter_map(|t| {
                                        if g_tys.contains(t) {
                                            Some(t.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                let g_ty = TypeVariant::Generic(filtered_tys);
                                TypeVariant::List(Box::new(g_ty))
                            }
                            c_ty => {
                                if list_tys.contains(c_ty) {
                                    TypeVariant::List(Box::new(c_ty.clone()))
                                } else {
                                    contract.diagnostics.push(Report::type_error(
                                        loc.clone(),
                                        String::from(
                                            "Functional's return list type cannot be derived.",
                                        ),
                                    ));
                                    return Err(());
                                }
                            }
                        }
                    }
                    // same as for generic, but encapsulated inside set type.
                    // If the list type is concrete, then we return the concrete type.
                    TypeVariant::Set(l_ty) => {
                        let list_tys: Vec<TypeVariant> = tys
                            .iter()
                            .filter_map(|t| {
                                if let TypeVariant::Set(l) = t {
                                    Some(l.as_ref().clone())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        match l_ty.as_ref() {
                            TypeVariant::Generic(g_tys) => {
                                let filtered_tys = list_tys
                                    .iter()
                                    .filter_map(|t| {
                                        if g_tys.contains(t) {
                                            Some(t.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                let g_ty = TypeVariant::Generic(filtered_tys);
                                TypeVariant::Set(Box::new(g_ty))
                            }
                            c_ty => {
                                if list_tys.contains(c_ty) {
                                    TypeVariant::Set(Box::new(c_ty.clone()))
                                } else {
                                    contract.diagnostics.push(Report::type_error(
                                        loc.clone(),
                                        String::from(
                                            "Functional's set list type cannot be derived.",
                                        ),
                                    ));
                                    return Err(());
                                }
                            }
                        }
                    }
                    // if function return type, then we check that it is in the list of allowed
                    // types.
                    c_ty => {
                        if tys.contains(c_ty) {
                            c_ty.clone()
                        } else {
                            return Err(());
                        }
                    }
                }
            }
        }
        // if the expected type is none, we just ignore the return type of the function call.
        ExpectedType::Empty => func.return_ty.ty().clone(),
    };

    Ok(Expression::FunctionCall(FunctionCall {
        loc: loc.clone(),
        sym: symbol.clone(),
        args: parsed_args,
        returns: return_ty.clone(),
    }))
}

/// Resolve member access.
///
/// # Note
/// Currently only variables are supported.
/// - Check that the var and declaration exist.
/// - Check that the member exists.
/// - Check the type match.
pub fn resolve_member_access(
    expr: &parsed_ast::Expression,
    member: &Identifier,
    loc: Span,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    if let parsed_ast::Expression::Variable(_) = expr {
        let resolved_expr = expression(expr, ExpectedType::Dynamic(vec![]), scope, contract)?;
        let ast::Expression::Variable(var) = &resolved_expr else {
            return Err(());
        };

        let (mty, pos) = match &var.ty {
            TypeVariant::State(s) => {
                let state_decl = &contract.states[s.i].clone();
                if state_decl.body.is_some() {
                    let members = state_decl.fields(contract);

                    if let Some(pos) = members.iter().position(|m| m.name.name == member.name) {
                        let field = &members[pos];
                        let ty = field.ty.ty.clone();
                        (ty, pos)
                    } else {
                        contract.diagnostics.push(Report::semantic_error(
                            member.loc.clone(),
                            String::from("Member does not exist"),
                        ));
                        return Err(());
                    }
                } else {
                    contract.diagnostics.push(Report::semantic_error(
                        loc.clone(),
                        String::from("This state has no members."),
                    ));
                    return Err(());
                }
            }
            TypeVariant::Struct(s) => {
                let state_decl = &contract.structs[s.i];
                let members = &state_decl.fields;

                if let Some(pos) = members.iter().position(|m| m.name.name == member.name) {
                    let field = &members[pos];
                    let ty = field.ty.ty.clone();
                    (ty, pos)
                } else {
                    contract.diagnostics.push(Report::semantic_error(
                        member.loc.clone(),
                        String::from("Member does not exist"),
                    ));
                    return Err(());
                }
            }
            TypeVariant::Model(s) => {
                let members = contract.models[s.i].fields(contract);

                if let Some(pos) = members.iter().position(|m| m.name.name == member.name) {
                    let field = &members[pos];
                    let ty = field.ty.ty.clone();
                    (ty, pos)
                } else {
                    contract.diagnostics.push(Report::semantic_error(
                        member.loc.clone(),
                        String::from("Member does not exist"),
                    ));
                    return Err(());
                }
            }
            TypeVariant::Enum(s) => {
                let state_decl = &contract.enums[s.i];
                let members: &Vec<&String> = &state_decl.variants.keys().collect();

                if let Some(pos) = &members.iter().position(|m| *m == &member.name) {
                    let ty = TypeVariant::Enum(s.clone());
                    (ty, *pos)
                } else {
                    contract.diagnostics.push(Report::semantic_error(
                        member.loc.clone(),
                        String::from("Member does not exist"),
                    ));
                    return Err(());
                }
            }
            _ => {
                contract.diagnostics.push(Report::semantic_error(
                    loc.clone(),
                    String::from("This type does not support member access."),
                ));
                return Err(());
            }
        };

        let ty = match &expected_ty {
            ExpectedType::Concrete(ty) => {
                if ty != &mty {
                    report_type_mismatch(&expected_ty, &[mty], &loc, contract);
                    return Err(());
                }
                mty
            }
            ExpectedType::Dynamic(tys) => {
                if !tys.contains(&mty) && !tys.is_empty() {
                    report_type_mismatch(&expected_ty, &[mty], &loc, contract);
                    return Err(());
                } else {
                    mty
                }
            }
            ExpectedType::Empty => {
                contract.diagnostics.push(Report::semantic_error(
                    loc,
                    String::from("Member access can only be used in expressions or statements."),
                ));
                return Err(());
            }
        };

        Ok(Expression::MemberAccess(MemberAccess {
            loc: loc.clone(),
            expr: Box::new(resolved_expr),
            member: (pos, member.loc.clone()),
            ty,
        }))
    } else {
        contract.diagnostics.push(Report::semantic_error(
            loc.clone(),
            String::from("Non variable access is currently unsupported"),
        ));
        Err(())
    }
}

/// Resolve piping. We simply convert to a nested function call.
/// # Errors
/// - Rhs expression is not a function call.
pub fn resolve_pipe(
    left: &parsed_ast::Expression,
    right: &parsed_ast::Expression,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    // check that the rhs expression is a function.
    let parsed_ast::Expression::FunctionCall(f_call) = right else {
        contract.diagnostics.push(Report::semantic_error(
            right.loc().clone(),
            String::from("Expression must be function"),
        ));
        return Err(());
    };

    // simply prepend expression to the list of arguments.
    let mut f_call = f_call.clone();
    f_call.args.insert(0, left.clone());

    expression(
        &parsed_ast::Expression::FunctionCall(f_call),
        expected_ty,
        scope,
        contract,
    )
}

/// Resolve initialise of the structure type.
/// # Note
/// - Auto-object fill is currently unsupported.
/// # Errors
/// - The type of the structure mismatches the expected one.
/// - Invalid number of type of arguments.
pub fn resolve_struct_init(
    ident: &Identifier,
    args: &[parsed_ast::Expression],
    auto_object: &Option<Identifier>,
    loc: Span,
    contract: &mut ContractDefinition,
    scope: &mut Scope,
    expected_ty: ExpectedType,
) -> Result<Expression, ()> {
    if auto_object.is_some() {
        // todo: implement auto-object
        contract.diagnostics.push(Report::semantic_error(
            loc.clone(),
            String::from("Auto-object is currently unsupported."),
        ));
        return Err(());
    }
    let Some(sym) = GlobalSymbol::lookup(contract, ident) else {
        return Err(());
    };

    let resolve_model = |s: &SymbolInfo,
                         scope: &mut Scope,
                         contract: &mut ContractDefinition|
     -> Result<(Vec<Expression>, Option<SymbolInfo>), ()> {
        let model_decl = contract.models[s.i].clone();
        let fields = &model_decl.fields(contract);
        let parent = model_decl.parent;

        if fields.len() != args.len() {
            report_mismatched_args_len(&loc, fields.len(), args.len(), contract);
            return Err(());
        }
        let (parsed_args, error_args) = parse_args(args, fields, scope, contract);

        if error_args {
            contract.diagnostics.push(Report::type_error(
                loc.clone(),
                String::from("Argument mismatched."),
            ));
            return Err(());
        }
        Ok((parsed_args, parent))
    };

    let check_types = |tv: TypeVariant, contract: &mut ContractDefinition| -> Result<(), ()> {
        match &expected_ty {
            ExpectedType::Empty => {
                contract.diagnostics.push(Report::type_error(
                    loc.clone(),
                    String::from("Initialisation can only happen in variable declaration."),
                ));
                Err(())
            }
            ExpectedType::Concrete(ty) if ty != &tv => {
                report_type_mismatch(&expected_ty, &[tv], &loc, contract);
                Err(())
            }
            ExpectedType::Dynamic(tys) if !tys.contains(&tv) && !tys.is_empty() => {
                report_type_mismatch(&expected_ty, &[tv], &loc, contract);
                Err(())
            }
            _ => Ok(()),
        }
    };

    match sym {
        GlobalSymbol::Struct(s) => {
            check_types(TypeVariant::Struct(s.clone()), contract)?;

            let struct_decl = contract.structs[s.i].clone();
            if struct_decl.fields.len() != args.len() {
                report_mismatched_args_len(&loc, struct_decl.fields.len(), args.len(), contract);
                return Err(());
            }
            let (parsed_args, error_args) = parse_args(args, &struct_decl.fields, scope, contract);

            if error_args {
                contract.diagnostics.push(Report::type_error(
                    loc.clone(),
                    String::from("Argument types mismatched."),
                ));
                return Err(());
            }

            Ok(Expression::StructInit(StructInit {
                loc: loc.clone(),
                name: ident.clone(),
                args: parsed_args,
                auto_object: None,
                parent: None,
                ty: TypeVariant::Struct(s.clone()),
            }))
        }
        GlobalSymbol::Model(s) => {
            check_types(TypeVariant::Model(s.clone()), contract)?;

            let (parsed_args, parent) = resolve_model(&s, scope, contract)?;

            Ok(Expression::StructInit(StructInit {
                loc: loc.clone(),
                name: ident.clone(),
                args: parsed_args,
                auto_object: None,
                parent,
                ty: TypeVariant::Model(s.clone()),
            }))
        }
        GlobalSymbol::State(s) => {
            check_types(TypeVariant::State(s.clone()), contract)?;

            let state_decl = contract.states[s.i].clone();
            if state_decl.body.is_none() {
                if !args.is_empty() {
                    contract.diagnostics.push(Report::semantic_error(
                        loc.clone(),
                        String::from("This state has no body to initialise."),
                    ));
                    return Err(());
                } else {
                    return Ok(Expression::StructInit(StructInit {
                        loc: loc.clone(),
                        name: ident.clone(),
                        args: vec![],
                        auto_object: None,
                        parent: None,
                        ty: TypeVariant::State(s.clone()),
                    }));
                };
            }

            let body = &state_decl.body.unwrap();
            let (parsed_args, parent) = match body {
                StateBody::Raw(fields) => {
                    if fields.len() != args.len() {
                        report_mismatched_args_len(&loc, fields.len(), args.len(), contract);
                        return Err(());
                    }
                    let (parsed_args, error_args) = parse_args(args, fields, scope, contract);

                    if error_args {
                        contract.diagnostics.push(Report::type_error(
                            loc.clone(),
                            String::from("Argument types mismatched."),
                        ));
                        return Err(());
                    }
                    (parsed_args, None)
                }
                StateBody::Model(s) => {
                    // todo: support destructuring of fields.
                    // if we have a single argument, then it is probably a model var.
                    if args.len() == 1 {
                        let attempted_expr = expression(
                            &args[0],
                            ExpectedType::Concrete(TypeVariant::Model(s.clone())),
                            scope,
                            contract,
                        );
                        if let Ok(Expression::Variable(var)) = attempted_expr {
                            return Ok(Expression::StructInit(StructInit {
                                loc: loc.clone(),
                                name: ident.clone(),
                                args: vec![],
                                auto_object: Some(var.element),
                                parent: None,
                                ty: TypeVariant::State(s.clone()),
                            }));
                        } else {
                            resolve_model(s, scope, contract)?
                        }
                    } else {
                        resolve_model(s, scope, contract)?
                    }
                }
            };
            Ok(Expression::StructInit(StructInit {
                loc: loc.clone(),
                name: ident.clone(),
                args: parsed_args,
                auto_object: None,
                parent,
                ty: TypeVariant::State(s.clone()),
            }))
        }
        GlobalSymbol::Function(_) | GlobalSymbol::Enum(_) => {
            contract.diagnostics.push(Report::semantic_error(
                ident.loc.clone(),
                String::from("Functions, States and Enums be initialised."),
            ));
            Err(())
        }
    }
}

fn parse_args(
    args: &[parsed_ast::Expression],
    params: &[Param],
    scope: &mut Scope,
    contract: &mut ContractDefinition,
) -> (Vec<Expression>, bool) {
    let mut error_args = false;
    let parsed_args: Vec<Expression> = args
        .iter()
        .zip(params.iter())
        .filter_map(|(e, p)| {
            // if the param is generic, then we convert it to the dynamic expected type.
            let arg_expected_ty = match &p.ty.ty {
                TypeVariant::Generic(tys) => ExpectedType::Dynamic(tys.clone()),
                a_ty => ExpectedType::Concrete(a_ty.clone()),
            };
            if let Ok(res_arg) = expression(e, arg_expected_ty, scope, contract) {
                Some(res_arg)
            } else {
                error_args = true;
                None
            }
        })
        .collect();
    (parsed_args, error_args)
}

fn check_func_return_type(ty: &TypeVariant, return_ty: &TypeVariant) -> bool {
    if let TypeVariant::List(l_ty) = return_ty {
        match l_ty.as_ref() {
            TypeVariant::Generic(allowed_tys) => {
                for at in allowed_tys {
                    if check_func_return_type(ty, at) {
                        return true;
                    }
                }
                false
            }
            a_ty => check_func_return_type(ty, a_ty),
        }
    } else if let TypeVariant::Set(l_ty) = return_ty {
        match l_ty.as_ref() {
            TypeVariant::Generic(allowed_tys) => {
                for at in allowed_tys {
                    if check_func_return_type(ty, at) {
                        return true;
                    }
                }
                false
            }
            a_ty => check_func_return_type(ty, a_ty),
        }
    } else if let TypeVariant::Generic(allowed_tys) = return_ty {
        for at in allowed_tys {
            if check_func_return_type(ty, at) {
                return true;
            }
        }
        false
    } else if let TypeVariant::List(l_ty) = ty {
        match l_ty.as_ref() {
            TypeVariant::Generic(allowed_tys) => {
                for at in allowed_tys {
                    if check_func_return_type(at, return_ty) {
                        return true;
                    }
                }
                false
            }
            a_ty => check_func_return_type(a_ty, return_ty),
        }
    } else if let TypeVariant::Set(l_ty) = ty {
        match l_ty.as_ref() {
            TypeVariant::Generic(allowed_tys) => {
                for at in allowed_tys {
                    if check_func_return_type(at, return_ty) {
                        return true;
                    }
                }
                false
            }
            a_ty => check_func_return_type(a_ty, return_ty),
        }
    } else if let TypeVariant::Generic(allowed_tys) = ty {
        for at in allowed_tys {
            if check_func_return_type(at, return_ty) {
                return true;
            }
        }
        false
    } else {
        ty == return_ty
    }
}

fn report_mismatched_args_len(
    loc: &Span,
    expected: usize,
    got: usize,
    contract: &mut ContractDefinition,
) {
    contract.diagnostics.push(Report::semantic_error(
        loc.clone(),
        format!(
            "Invalid number of arguments. Expected {}, got {}",
            expected.green().bold(),
            got.red().bold()
        ),
    ));
}
