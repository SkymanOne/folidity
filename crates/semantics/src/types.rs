use std::collections::HashSet;

use crate::ast::{List, Mapping, Param, Set, Type, TypeVariant};
use crate::contract::ContractDefinition;
use crate::global_symbol::{GlobalSymbol, SymbolInfo};
use folidity_diagnostics::Report;
use folidity_parser::ast::Identifier;
use folidity_parser::{ast as parsed_ast, Span};
use petgraph::algo::{all_simple_paths, tarjan_scc};
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph};

type FieldGraph = Graph<(), usize, Directed, usize>;

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

/// Attempts to find a user defined type recursion.
/// Returns span of the of the first instance.
///
/// # Outline
/// - Generate a dependency tree of user defined types.
/// - Check for cycles.
/// # Note
/// Inspired by https://github.com/hyperledger/solang/blob/d7a875afe73f95e3c9d5112aa36c8f9eb91a6e00/src/sema/types.rs#L359.
///
/// Licensed as Apache 2.0
//todo: rewrite.
//TODO: support finite size recursive types.
pub fn find_user_type_recursion(contract: &mut ContractDefinition) -> Result<(), Span> {
    let mut edges = HashSet::new();
    for n in 0..contract.structs.len() {
        collect_edges(
            &mut edges,
            &contract.structs[n].fields,
            n,
            &mut contract.clone(),
        )
    }

    let graph: FieldGraph = Graph::from_edges(edges);
    let tarjan = tarjan_scc(&graph);
    let mut nodes = HashSet::new();
    for node in tarjan.iter().flatten() {
        nodes.insert(node);
    }

    for node in nodes {
        check_for_recursive_types(node.index(), &graph, contract);
    }

    for n in 0..contract.structs.len() {
        for field in contract.structs[n].fields.iter().filter(|f| f.recursive) {
            contract.diagnostics.push(Report::semantic_error(
                field.loc.clone(),
                String::from("Recursive field detected."),
            ));
        }
    }

    Ok(())
}

/// Collect field dependencies into the graph edges.
fn collect_edges(
    edges: &mut HashSet<(usize, usize, usize)>,
    fields: &[Param],
    struct_no: usize,
    contract: &mut ContractDefinition,
) {
    for (no, field) in fields.iter().enumerate() {
        for dependency in field.ty.custom_type_dependencies(contract) {
            if edges.insert((no, dependency, struct_no)) {
                collect_edges(edges, fields, struct_no, contract)
            }
        }
    }
}

/// Check for recursive edges.
fn check_for_recursive_types(node: usize, graph: &FieldGraph, contract: &mut ContractDefinition) {
    for n in 0..contract.structs.len() {
        for simple_path in
            all_simple_paths::<Vec<_>, &FieldGraph>(graph, n.into(), node.into(), 0, None)
        {
            for (a, b) in simple_path.windows(2).map(|pair| (pair[0], pair[1])) {
                for edge in graph.edges_connecting(a, b) {
                    contract.structs[a.index()].fields[*edge.weight()].recursive = true;
                }
            }
        }
    }
}
