use std::collections::HashSet;

use crate::ast::{List, Mapping, Param, Set, StateBody, Type, TypeVariant};
use crate::contract::ContractDefinition;
use crate::global_symbol::GlobalSymbol;
use folidity_diagnostics::Report;
use folidity_parser::ast as parsed_ast;
use petgraph::algo::{all_simple_paths, tarjan_scc};
use petgraph::{Directed, Graph};

type FieldGraph = Graph<(), usize, Directed, usize>;

#[derive(Debug, Clone)]
pub struct DelayedDeclaration<T> {
    pub decl: T,
    pub i: usize,
}

/// Saved declaration for later analysis.
/// The first pass should resolve the fields.
/// The second pass should resolve model bounds.
#[derive(Debug, Default)]
pub struct DelayedDeclarations {
    pub structs: Vec<DelayedDeclaration<folidity_parser::ast::StructDeclaration>>,
    pub models: Vec<DelayedDeclaration<folidity_parser::ast::ModelDeclaration>>,
    pub states: Vec<DelayedDeclaration<folidity_parser::ast::StateDeclaration>>,
}

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
pub fn find_user_type_recursion(contract: &mut ContractDefinition) {
    let mut edges = HashSet::new();
    for n in 0..contract.structs.len() {
        collect_edges(&mut edges, &contract.structs[n].fields, n)
    }

    let graph: FieldGraph = Graph::from_edges(edges);
    let tarjan = tarjan_scc(&graph);
    let mut nodes = HashSet::new();
    for node in tarjan.iter().flatten() {
        nodes.insert(node);
    }

    for node in nodes {
        check_for_recursive_fields(node.index(), &graph, contract);
    }

    for n in 0..contract.structs.len() {
        for field in contract.structs[n].fields.iter().filter(|f| f.recursive) {
            contract.diagnostics.push(Report::semantic_error(
                field.loc.clone(),
                String::from("Recursive field detected."),
            ));
        }
    }
}

/// Collect field dependencies into the graph edges.
fn collect_edges(edges: &mut HashSet<(usize, usize, usize)>, fields: &[Param], struct_no: usize) {
    for (no, field) in fields.iter().enumerate() {
        for dependency in field.ty.custom_type_dependencies() {
            if edges.insert((no, dependency, struct_no)) {
                collect_edges(edges, fields, struct_no)
            }
        }
    }
}

/// Check for recursive edges.
fn check_for_recursive_fields(node: usize, graph: &FieldGraph, contract: &mut ContractDefinition) {
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

/// Validate that fields of user defined types do not contain references to models and states.
pub fn validate_fields(contract: &mut ContractDefinition) {
    let mut validate = |fields: &[Param]| {
        for field in fields.iter() {
            match &field.ty.ty {
                TypeVariant::Function(_) => contract.diagnostics.push(Report::semantic_error(
                    field.loc.clone(),
                    String::from("Function cannot be used as a field type."),
                )),
                TypeVariant::Model(_) => contract.diagnostics.push(Report::semantic_error(
                    field.loc.clone(),
                    String::from("Model cannot be used as a field type."),
                )),
                TypeVariant::State(_) => contract.diagnostics.push(Report::semantic_error(
                    field.loc.clone(),
                    String::from("State cannot be used as a field type."),
                )),
                _ => {}
            }
        }
    };

    for s in &contract.structs {
        validate(&s.fields);
    }

    for s in &contract.states {
        if let Some(StateBody::Raw(fields)) = &s.body {
            validate(fields);
        }
    }

    for m in &contract.models {
        validate(&m.fields);
    }
}

/// Check that model and state inheritance is valid.
pub fn check_inheritance(contract: &mut ContractDefinition, delay: &DelayedDeclarations) {
    for model in &delay.models {
        if let Some(ident) = &model.decl.parent {
            if let Some(symbol) = contract.declaration_symbols.get(&ident.name) {
                match symbol {
                    GlobalSymbol::Model(s) => contract.models[model.i].parent = Some(s.i),
                    _ => {
                        contract.diagnostics.push(Report::semantic_error(
                            ident.loc.clone(),
                            String::from("Model can only inherit other models."),
                        ));
                    }
                }
            } else {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    String::from("Model is not declared."),
                ));
            }
        }
    }

    for state in &delay.states {
        if let Some((ident, var)) = &state.decl.from {
            if let Some(symbol) = contract.declaration_symbols.get(&ident.name) {
                match symbol {
                    GlobalSymbol::State(s) => {
                        contract.states[state.i].from = Some((s.i, var.clone()))
                    }
                    _ => contract.diagnostics.push(Report::semantic_error(
                        ident.loc.clone(),
                        String::from("Declaration must a declared state."),
                    )),
                }
            } else {
                contract.diagnostics.push(Report::semantic_error(
                    ident.loc.clone(),
                    String::from("State is not declared."),
                ));
            }
        }
    }

    detect_model_cycle(contract);
    detect_state_cycle(contract);
}

/// Detect cyclic model inheritances.
fn detect_model_cycle(contract: &mut ContractDefinition) {
    let mut edges = HashSet::new();
    for edge in contract.models.iter().filter_map(|m| m.parent).enumerate() {
        edges.insert(edge);
    }
    let graph: FieldGraph = Graph::from_edges(edges);
    let tarjan = tarjan_scc(&graph);
    let mut nodes = HashSet::new();
    for node in tarjan.iter().flatten() {
        nodes.insert(node);
    }

    for node in nodes {
        for n in 0..contract.models.len() {
            for simple_path in all_simple_paths::<Vec<_>, &FieldGraph>(
                &graph,
                n.into(),
                node.index().into(),
                0,
                None,
            ) {
                for (a, b) in simple_path.windows(2).map(|p| (p[0], p[1])) {
                    contract.models[a.index()].recursive_parent = true;
                }
            }
        }
    }

    for model in contract.models.iter().filter(|m| m.recursive_parent) {
        contract.diagnostics.push(Report::semantic_error(
            model.loc.clone(),
            String::from("This model inheritance is cyclic."),
        ));
    }
}

/// Detect cyclic state transition bounds.
fn detect_state_cycle(contract: &mut ContractDefinition) {
    let mut edges = HashSet::new();
    for edge in contract
        .states
        .iter()
        .filter_map(|m| m.from.as_ref().map(|x| x.0))
        .enumerate()
    {
        edges.insert(edge);
    }
    let graph: FieldGraph = Graph::from_edges(edges);
    let tarjan = tarjan_scc(&graph);
    let mut nodes = HashSet::new();
    for node in tarjan.iter().flatten() {
        nodes.insert(node);
    }

    for node in nodes {
        for n in 0..contract.states.len() {
            for simple_path in all_simple_paths::<Vec<_>, &FieldGraph>(
                &graph,
                n.into(),
                node.index().into(),
                0,
                None,
            ) {
                for (a, _) in simple_path.windows(2).map(|p| (p[0], p[1])) {
                    contract.states[a.index()].recursive_parent = true;
                }
            }
        }
    }

    for state in contract.states.iter().filter(|m| m.recursive_parent) {
        contract.diagnostics.push(Report::semantic_error(
            state.loc.clone(),
            String::from("This state transition bound is cyclic."),
        ));
    }
}
