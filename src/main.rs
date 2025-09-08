use petgraph::Undirected;

use crate::models::graph_models::Node;
use crate::models::graph_models::Runtime;

mod models;

fn main() {
    let mut runtime = Runtime::<&str, Undirected>::new("test");
    let _ = runtime.add_connection(
        Node::new("person1", "filipe", vec![], None, "description of node1"),
        Node::new("person2", "maria", vec![], None, "description of node2"),
        "friendship",
        "description of friendship",
    );
    println!("{:?}", runtime.edges);
    println!("{:?}", runtime.nodes);
}
