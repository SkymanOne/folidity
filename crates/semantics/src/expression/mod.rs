mod literals;
mod nums;

use folidity_parser::ast as parsed_ast;

use crate::{
    ast::{Expression, TypeVariant},
    contract::ContractDefinition,
    symtable::SymTable,
    types::ExpectedType,
};

use self::{
    literals::{resolve_address, resolve_bool, resolve_char, resolve_hex, resolve_string},
    nums::{resolve_float, resolve_integer},
};

/// Resolve parsed expression to a concrete expression.
pub fn expression(
    expr: &parsed_ast::Expression,
    expected_ty: ExpectedType,
    symtable: &mut SymTable,
    contract: &mut ContractDefinition,
) -> Result<Expression, ()> {
    match expr {
        parsed_ast::Expression::Number(n) => {
            resolve_integer(&n.element, n.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Float(n) => {
            resolve_float(&n.element, n.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Hex(h) => {
            resolve_hex(&h.element, h.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Char(c) => {
            resolve_char(c.element, c.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::String(s) => {
            resolve_string(s.element.clone(), s.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Boolean(b) => {
            resolve_bool(b.element, b.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Address(a) => {
            resolve_address(&a.element, a.loc.clone(), contract, expected_ty)
        }
        parsed_ast::Expression::Variable(_) => todo!(),
        parsed_ast::Expression::Multiply(_) => todo!(),
        parsed_ast::Expression::Divide(_) => todo!(),
        parsed_ast::Expression::Modulo(_) => todo!(),
        parsed_ast::Expression::Add(_) => todo!(),
        parsed_ast::Expression::Subtract(_) => todo!(),
        parsed_ast::Expression::Equal(_) => todo!(),
        parsed_ast::Expression::NotEqual(_) => todo!(),
        parsed_ast::Expression::Greater(_) => todo!(),
        parsed_ast::Expression::Less(_) => todo!(),
        parsed_ast::Expression::GreaterEq(_) => todo!(),
        parsed_ast::Expression::LessEq(_) => todo!(),
        parsed_ast::Expression::In(_) => todo!(),
        parsed_ast::Expression::Not(_) => todo!(),
        parsed_ast::Expression::Or(_) => todo!(),
        parsed_ast::Expression::And(_) => todo!(),
        parsed_ast::Expression::FunctionCall(_) => todo!(),
        parsed_ast::Expression::MemberAccess(_) => todo!(),
        parsed_ast::Expression::Pipe(_) => todo!(),
        parsed_ast::Expression::StructInit(_) => todo!(),
        parsed_ast::Expression::List(_) => todo!(),
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
