mod complex;
mod eval;
mod literals;
mod nums;
mod ops;
#[cfg(test)]
mod tests;

use folidity_parser::ast::{
    self as parsed_ast,
};

use crate::{
    ast::{
        Expression,
        Param,
        TypeVariant,
    },
    contract::ContractDefinition,
    global_symbol::SymbolInfo,
    symtable::Scope,
    types::ExpectedType,
};

use self::{
    complex::{
        resolve_func_call,
        resolve_member_access,
        resolve_pipe,
        resolve_struct_init,
        resolve_variable,
    },
    literals::{
        resolve_address,
        resolve_bool,
        resolve_char,
        resolve_hex,
        resolve_lists,
        resolve_string,
    },
    nums::{
        resolve_float,
        resolve_integer,
    },
    ops::{
        resolve_addition,
        resolve_and,
        resolve_division,
        resolve_equality,
        resolve_greater,
        resolve_greater_eq,
        resolve_in,
        resolve_inequality,
        resolve_less,
        resolve_less_eq,
        resolve_modulo,
        resolve_multiply,
        resolve_not,
        resolve_or,
        resolve_subtraction,
    },
};

/// Resolve parsed expression to a concrete expression.
pub fn expression(
    expr: &parsed_ast::Expression,
    expected_ty: ExpectedType,
    scope: &mut Scope,
    contract: &mut ContractDefinition,
) -> Result<Expression, ()> {
    match expr {
        // literals
        parsed_ast::Expression::Number(u) => {
            resolve_integer(&u.element, u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Float(u) => {
            resolve_float(&u.element, u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Hex(u) => {
            resolve_hex(&u.element, u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Char(u) => {
            resolve_char(u.element, u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::String(u) => {
            resolve_string(u.element.clone(), u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Boolean(u) => {
            resolve_bool(u.element, u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Address(u) => {
            resolve_address(&u.element, u.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::List(u) => {
            resolve_lists(&u.element, u.loc.clone(), contract, scope, expected_ty)
        }
        // operations
        parsed_ast::Expression::Multiply(b) => {
            resolve_multiply(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Divide(b) => {
            resolve_division(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Modulo(b) => {
            resolve_modulo(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Add(b) => {
            resolve_addition(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Subtract(b) => {
            resolve_subtraction(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Equal(b) => {
            resolve_equality(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::NotEqual(b) => {
            resolve_inequality(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Greater(b) => {
            resolve_greater(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Less(b) => {
            resolve_less(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::GreaterEq(b) => {
            resolve_greater_eq(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::LessEq(b) => {
            resolve_less_eq(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::And(b) => {
            resolve_and(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Or(b) => {
            resolve_or(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Not(u) => {
            resolve_not(&u.element, u.loc.clone(), scope, contract, expected_ty)
        }
        parsed_ast::Expression::In(b) => {
            resolve_in(
                &b.left,
                &b.right,
                b.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        // complex expressions
        parsed_ast::Expression::Variable(ident) => {
            resolve_variable(ident, scope, contract, expected_ty)
        }
        parsed_ast::Expression::FunctionCall(f_call) => {
            resolve_func_call(
                &f_call.name,
                &f_call.args,
                f_call.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::MemberAccess(m_a) => {
            resolve_member_access(
                &m_a.expr,
                &m_a.member,
                m_a.loc.clone(),
                scope,
                contract,
                expected_ty,
            )
        }
        parsed_ast::Expression::Pipe(b) => {
            resolve_pipe(&b.left, &b.right, scope, contract, expected_ty)
        }
        parsed_ast::Expression::StructInit(s) => {
            resolve_struct_init(
                &s.name,
                &s.args,
                &s.auto_object,
                s.loc.clone(),
                contract,
                scope,
                expected_ty,
            )
        }
    }
}

/// Derives a concrete expected type from the list of supported types.
/// The first element is used as a fallback option in case `tys` is empty.
pub fn dynamic_to_concrete_type(tys: &[TypeVariant], allowed: &[TypeVariant]) -> ExpectedType {
    let mut allowed_tys = tys.iter().filter(|ty| allowed.contains(ty));
    if let Some(ty) = allowed_tys.next() {
        ExpectedType::Concrete(ty.clone())
    } else {
        ExpectedType::Concrete(allowed[0].clone())
    }
}

pub fn resolve_nested_fields(
    s: &Option<SymbolInfo>,
    fields: &mut Vec<Param>,
    contract: &ContractDefinition,
) {
    if let Some(parent_sym) = s {
        let parent_decl_fields = contract.models[parent_sym.i].fields.clone();
        let grand_parent = contract.models[parent_sym.i].parent.clone();
        resolve_nested_fields(&grand_parent, fields, contract);
        fields.extend(parent_decl_fields);
    }
}
