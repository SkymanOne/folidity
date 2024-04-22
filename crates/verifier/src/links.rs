use folidity_semantics::GlobalSymbol;
use petgraph::{
    algo::tarjan_scc,
    Graph,
    Undirected,
};
use std::collections::HashSet;

use crate::{
    ast::Constraint,
    SymbolicExecutor,
};

type LinkGraph = Graph<usize, usize, Undirected, usize>;

fn find_link_components(executor: &SymbolicExecutor) -> Vec<Vec<usize>> {
    let mut edges: HashSet<(usize, usize)> = HashSet::new();

    for (sym, d) in &executor.declarations {
        if d.links.is_empty() {
            continue;
        }
        let origin = executor
            .declarations
            .get_index_of(sym)
            .expect("should exist");

        for l in &d.links {
            edges.insert((origin, *l));
        }
    }

    // collect into a graph.
    let graph: LinkGraph = LinkGraph::from_edges(&edges);
    // since the graph is undirected, tarjan algo will just return sets of connected nodes.
    let components = tarjan_scc(&graph)
        .iter()
        .map(|vec| vec.iter().map(|x| x.index()).collect())
        .collect();

    components
}

pub fn build_constraint_blocks<'ctx>(
    executor: &mut SymbolicExecutor<'ctx>,
) -> Vec<Vec<(Constraint<'ctx>, GlobalSymbol)>> {
    let components = find_link_components(executor);
    let mut blocks = vec![];
    for decls in &components {
        let mut constraints: Vec<(Constraint, GlobalSymbol)> = vec![];
        for i in decls {
            let (sym, decl) = executor.declarations.get_index(*i).expect("should exist");

            for (_, c) in decl.constraints.clone() {
                constraints.push((c, sym.clone()));
            }
        }
        blocks.push(constraints);
    }
    blocks
}
