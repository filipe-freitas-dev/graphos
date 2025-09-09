use petgraph::Undirected;

use crate::models::graph_models::Node;
use crate::models::graph_models::Runtime;

mod models;

fn main() {
    let mut runtime = Runtime::<String, Undirected>::new("test");
    let _ = runtime.add_connection(
        Node::new(
            "person1",
            "filipe".to_string(),
            vec![],
            "description of node1",
        ),
        Node::new(
            "person2",
            "maria".to_string(),
            vec![],
            "description of node2",
        ),
        "friendship",
        "description of friendship",
    );
    println!("Graph edges: {:#?}", runtime.edges);
    println!("Graph nodes: {:#?}", runtime.nodes);
    println!("Graph runtime references: {:#?}", runtime.runtime_ref);
}
