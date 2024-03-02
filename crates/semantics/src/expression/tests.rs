use folidity_parser::{ast as parsed_ast, Span};

use crate::{
    ast::TypeVariant, contract::ContractDefinition, symtable::SymTable, types::ExpectedType,
};

use super::expression;

#[test]
fn test_list() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut symtable = SymTable::default();
    let parsed_list = parsed_ast::Expression::List(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: vec![
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "1".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "2".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "3".to_string(),
            }),
        ],
    });
    let resolved_expr = expression(
        &parsed_list,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Int))),
        &mut symtable,
        &mut contract,
    );
    assert!(resolved_expr.is_ok());

    let resolved_expr_err = expression(
        &parsed_list,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Float))),
        &mut symtable,
        &mut contract,
    );
    assert!(resolved_expr_err.is_err())
}
