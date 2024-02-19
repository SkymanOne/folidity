use crate::ast::{List, Mapping, Set, Type, TypeVariant};
use crate::contract::ContractDefinition;
use crate::global_symbol::GlobalSymbol;
use folidity_diagnostics::Report;
use folidity_parser::ast as parsed_ast;
use folidity_parser::ast::Identifier;

/// Maps type from parsed AST to semantically resolved type.
/// - Primitive types are simply mapped 1-1.
/// - User defined types (e.g. structs, enums) are looked up in the global symbol table.
/// - List types are recursively mapped.
pub fn map_type(contract: &mut ContractDefinition, ty: &parsed_ast::Type) -> Result<Type, ()> {
    let variant = match &ty.ty {
        parsed_ast::TypeVariant::Int => TypeVariant::Int,
        parsed_ast::TypeVariant::Uint => TypeVariant::Uint,
        parsed_ast::TypeVariant::Float => TypeVariant::Float,
        parsed_ast::TypeVariant::Char => TypeVariant::Char,
        parsed_ast::TypeVariant::String => TypeVariant::String,
        parsed_ast::TypeVariant::Hex => TypeVariant::Hex,
        parsed_ast::TypeVariant::Address => TypeVariant::Address,
        parsed_ast::TypeVariant::Unit => TypeVariant::Address,
        parsed_ast::TypeVariant::Bool => TypeVariant::Bool,
        parsed_ast::TypeVariant::Set(s) => {
            let set_ty = map_type(contract, &s.ty)?;
            TypeVariant::Set(Set::new(Box::new(set_ty)))
        }
        parsed_ast::TypeVariant::List(l) => {
            let list_ty = map_type(contract, &l.ty)?;
            TypeVariant::List(List::new(Box::new(list_ty)))
        }
        parsed_ast::TypeVariant::Mapping(m) => {
            let m_from_ty = map_type(contract, &m.from_ty)?;
            let m_to_ty = map_type(contract, &m.to_ty)?;
            TypeVariant::Mapping(Mapping::new(
                Box::new(m_from_ty),
                m.relation.clone(),
                Box::new(m_to_ty),
            ))
        }
        parsed_ast::TypeVariant::Custom(user_ty) => {
            if let Some(symbol) = contract.declaration_symbols.get(&user_ty.name) {
                match symbol {
                    GlobalSymbol::Struct(info) => TypeVariant::Struct(info.clone()),
                    GlobalSymbol::Model(info) => TypeVariant::Model(info.clone()),
                    GlobalSymbol::Enum(info) => TypeVariant::Enum(info.clone()),
                    GlobalSymbol::State(info) => TypeVariant::State(info.clone()),
                    GlobalSymbol::Function(info) => TypeVariant::Function(info.clone()),
                }
            } else {
                contract.diagnostics.push(Report::semantic_error(
                    user_ty.loc.clone(),
                    format!("`{}` is not defined", user_ty.name),
                ));
                return Err(());
            }
        }
    };

    Ok(Type {
        loc: ty.loc.clone(),
        ty: variant,
    })
}
