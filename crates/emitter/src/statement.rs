use folidity_semantics::ast::Statement;

use crate::{
    ast::Chunk,
    expression::EmitExprArgs,
};

pub fn emit_statement(
    stmt: &Statement,
    chunks: &mut Vec<Chunk>,
    args: &mut EmitExprArgs,
) -> Result<(), ()> {
    match stmt {
        Statement::Variable(_) => todo!(),
        Statement::Assign(_) => todo!(),
        Statement::IfElse(_) => todo!(),
        Statement::ForLoop(_) => todo!(),
        Statement::Iterator(_) => todo!(),
        Statement::Return(_) => todo!(),
        Statement::Expression(_) => todo!(),
        Statement::StateTransition(_) => todo!(),
        Statement::Block(_) => todo!(),
        Statement::Skip(_) => todo!(),
        Statement::Error(_) => unreachable!(),
    }
}
