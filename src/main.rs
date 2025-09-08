use petgraph::Undirected;
use petgraph::graph::Graph;
use petgraph::visit::EdgeRef;

use crate::models::graph_models::Node;

mod models;

fn main() {
    let mut graph = Graph::<Node<String>, u32, Undirected>::new_undirected();

    let a = graph.add_node(Node::new(
        String::from("conteúdo 2"),
        vec![],
        "description1",
    ));
    let b = graph.add_node(Node::new(
        String::from("conteúdo 2"),
        vec![],
        "description1",
    ));
    let c = graph.add_node(Node::new(
        String::from("conteúdo 2"),
        vec![],
        "description1",
    ));
    let d = graph.add_node(Node::new(
        String::from("conteúdo 2"),
        vec![],
        "description1",
    ));

    graph.add_edge(a, b, 5);
    graph.add_edge(a, c, 3);
    graph.add_edge(c, d, 4);
    graph.add_edge(b, d, 2);

    println!("== Pesos originais ==");

    for edge_ref in graph.edge_references() {
        let source = edge_ref.source(); // NodeIndex
        let target = edge_ref.target(); // NodeIndex
        let weight = edge_ref.weight(); // &u32

        println!(
            "\"{:?}\" → \"{:?}\" | peso: {}",
            graph[source], graph[target], weight
        );
    }
}
