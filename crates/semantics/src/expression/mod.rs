mod nums;

use folidity_parser::ast as parsed_ast;

use crate::{
    ast::Expression, contract::ContractDefinition, symtable::SymTable, types::ExpectedType,
};

use self::nums::resolve_integer;

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
        parsed_ast::Expression::Variable(_) => todo!(),
        parsed_ast::Expression::Boolean(_) => todo!(),
        parsed_ast::Expression::Float(_) => todo!(),
        parsed_ast::Expression::String(_) => todo!(),
        parsed_ast::Expression::Char(_) => todo!(),
        parsed_ast::Expression::Hex(_) => todo!(),
        parsed_ast::Expression::Address(_) => todo!(),
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
