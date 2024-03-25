use std::collections::HashSet;

use crate::{
    ast::{
        Expression,
        FunctionType,
        Mapping,
        Param,
        StateBody,
        Type,
        TypeVariant,
    },
    contract::ContractDefinition,
    global_symbol::GlobalSymbol,
};
use folidity_diagnostics::{
    Color,
    Fmt,
    Report,
};
use folidity_parser::{
    ast as parsed_ast,
    Span,
};
use petgraph::{
    algo::{
        all_simple_paths,
        tarjan_scc,
    },
    Directed,
    Graph,
};

type FieldGraph = Graph<(), usize, Directed, usize>;

#[derive(Debug, Clone)]
pub struct DelayedDeclaration<T> {
    pub decl: T,
    pub i: usize,
}

/// Saved declaration for later analysis.
/// The first pass should resolve the fields.
/// The second pass should resolve model bounds.
#[derive(Debug)]
pub struct DelayedDeclarations {
    pub structs: Vec<DelayedDeclaration<parsed_ast::StructDeclaration>>,
    pub models: Vec<DelayedDeclaration<parsed_ast::ModelDeclaration>>,
    pub states: Vec<DelayedDeclaration<parsed_ast::StateDeclaration>>,
    pub functions: Vec<DelayedDeclaration<parsed_ast::FunctionDeclaration>>,
}

/// The expected type the expression is expected to resolve to.
#[derive(Debug, Clone)]
pub enum ExpectedType {
    /// The expression is not expected to resolve to any type (e.g. a function call)
    Empty,
    /// The expression is expected to resolve to a concrete type.
    /// e.g. `let a: int = <expr>`
    Concrete(TypeVariant),
    /// The expression can be resolved to different types and casted later.
    Dynamic(Vec<TypeVariant>),
}

impl ExpectedType {
    fn display(&self, contract: &ContractDefinition) -> String {
        match self {
            ExpectedType::Empty => "nothing".to_string(),
            ExpectedType::Concrete(ty) => ty.display(contract),
            ExpectedType::Dynamic(tys) => {
                let args = tys.iter().fold(String::new(), |acc, x| {
                    format!("{}, {}", acc, x.display(contract))
                });
                args.trim_start_matches(", ").to_string()
            }
        }
    }
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
        parsed_ast::TypeVariant::Unit => TypeVariant::Unit,
        parsed_ast::TypeVariant::Bool => TypeVariant::Bool,
        parsed_ast::TypeVariant::Set(s) => {
            let set_ty = map_type(contract, &s.ty)?;
            TypeVariant::Set(Box::new(set_ty.ty))
        }
        parsed_ast::TypeVariant::List(l) => {
            let list_ty = map_type(contract, &l.ty)?;
            TypeVariant::List(Box::new(list_ty.ty))
        }
        parsed_ast::TypeVariant::Mapping(m) => {
            let m_from_ty = map_type(contract, &m.from_ty)?;
            let m_to_ty = map_type(contract, &m.to_ty)?;
            TypeVariant::Mapping(Mapping::new(
                Box::new(m_from_ty.ty),
                m.relation.clone(),
                Box::new(m_to_ty.ty),
            ))
        }
        parsed_ast::TypeVariant::Custom(user_ty) => {
            if let Some(symbol) = GlobalSymbol::lookup(contract, user_ty) {
                match symbol {
                    GlobalSymbol::Struct(info) => TypeVariant::Struct(info.clone()),
                    GlobalSymbol::Model(info) => TypeVariant::Model(info.clone()),
                    GlobalSymbol::Enum(info) => TypeVariant::Enum(info.clone()),
                    GlobalSymbol::State(info) => TypeVariant::State(info.clone()),
                    GlobalSymbol::Function(info) => {
                        let i = info.i;
                        let func = contract.functions.get(i).unwrap();
                        let param_tys: Vec<TypeVariant> =
                            func.params.values().map(|p| &p.ty.ty).cloned().collect();
                        let return_ty = func.return_ty.ty().clone();
                        TypeVariant::Function(FunctionType {
                            params: param_tys,
                            returns: Box::new(return_ty),
                        })
                    }
                }
            } else {
                return Err(());
            }
        }
    };

    Ok(Type {
        loc: ty.loc.clone(),
        ty: variant,
    })
}

impl Expression {
    ///  Retrieve type from the expression.
    pub fn ty(&self) -> &TypeVariant {
        match self {
            Expression::Variable(e) => &e.ty,
            Expression::Int(e) => &e.ty,
            Expression::UInt(e) => &e.ty,
            Expression::Float(e) => &e.ty,
            Expression::Boolean(e) => &e.ty,
            Expression::String(e) => &e.ty,
            Expression::Char(e) => &e.ty,
            Expression::Hex(e) => &e.ty,
            Expression::Address(e) => &e.ty,
            Expression::Multiply(e) => &e.ty,
            Expression::Divide(e) => &e.ty,
            Expression::Modulo(e) => &e.ty,
            Expression::Add(e) => &e.ty,
            Expression::Subtract(e) => &e.ty,
            Expression::Equal(e) => &e.ty,
            Expression::NotEqual(e) => &e.ty,
            Expression::Greater(e) => &e.ty,
            Expression::Less(e) => &e.ty,
            Expression::GreaterEq(e) => &e.ty,
            Expression::LessEq(e) => &e.ty,
            Expression::In(e) => &e.ty,
            Expression::Not(e) => &e.ty,
            Expression::Or(e) => &e.ty,
            Expression::And(e) => &e.ty,
            Expression::FunctionCall(e) => &e.returns,
            Expression::MemberAccess(e) => &e.ty,
            Expression::StructInit(e) => &e.ty,
            Expression::List(e) => &e.ty,
            Expression::Enum(e) => &e.ty,
        }
    }
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
// todo: rewrite.
// TODO: support finite size recursive types.
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
        for dependency in field.ty.ty.custom_type_dependencies() {
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

/// Validate that fields of user defined types do not contain references to models and
/// states.
pub fn validate_fields(contract: &mut ContractDefinition) {
    let mut validate = |fields: &[Param]| {
        for field in fields.iter() {
            match &field.ty.ty {
                TypeVariant::Function(_) => {
                    contract.diagnostics.push(Report::semantic_error(
                        field.loc.clone(),
                        String::from("Function cannot be used as a field type."),
                    ))
                }
                TypeVariant::Model(_) => {
                    contract.diagnostics.push(Report::semantic_error(
                        field.loc.clone(),
                        String::from("Model cannot be used as a field type."),
                    ))
                }
                TypeVariant::State(_) => {
                    contract.diagnostics.push(Report::semantic_error(
                        field.loc.clone(),
                        String::from("State cannot be used as a field type."),
                    ))
                }
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
            if let Some(symbol) = GlobalSymbol::lookup(contract, ident) {
                match symbol {
                    GlobalSymbol::Model(s) => contract.models[model.i].parent = Some(s.clone()),
                    _ => {
                        contract.diagnostics.push(Report::semantic_error(
                            ident.loc.clone(),
                            String::from("Model can only inherit other models."),
                        ));
                    }
                }
            }
        }
    }

    for state in &delay.states {
        if let Some((ident, var)) = &state.decl.from {
            if let Some(symbol) = GlobalSymbol::lookup(contract, ident) {
                match symbol {
                    GlobalSymbol::State(s) => {
                        contract.states[state.i].from = Some((s, var.clone()))
                    }
                    _ => {
                        contract.diagnostics.push(Report::semantic_error(
                            ident.loc.clone(),
                            String::from("Declaration must a declared state."),
                        ))
                    }
                }
            }
        }
    }

    detect_model_cycle(contract);
    detect_state_cycle(contract);
}

/// Detect cyclic model inheritances.
fn detect_model_cycle(contract: &mut ContractDefinition) {
    let mut edges = HashSet::new();
    for edge in contract
        .models
        .iter()
        .enumerate()
        .filter_map(|(i, m)| m.parent.as_ref().map(|s| (s.i, i)))
    {
        edges.insert(edge);
    }
    let graph: FieldGraph = Graph::from_edges(edges);
    let tarjan = tarjan_scc(&graph);
    let mut nodes = HashSet::new();
    for node in tarjan.iter().filter(|nodes| nodes.len() > 1).flatten() {
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
                for (a, _) in simple_path.windows(2).map(|p| (p[0], p[1])) {
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
        .filter_map(|m| m.from.as_ref().map(|x| x.0.i))
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

/// Push diagnostic error about the type mismatch.
pub(super) fn report_type_mismatch(
    expected: &ExpectedType,
    actual: &[TypeVariant],
    loc: &Span,
    contract: &mut ContractDefinition,
) {
    let actual = actual.iter().fold(String::new(), |acc, x| {
        format!("{}, {}", acc, x.display(contract).fg(Color::Magenta))
    });
    contract.diagnostics.push(Report::type_error(
        loc.clone(),
        format!(
            "Mismatched types: expected to resolve to {}, but expression can only resolve to {}",
            expected.display(contract).fg(Color::Green),
            actual.trim_start_matches(", ")
        ),
    ));
}
