use petgraph::Undirected;

use crate::models::graph_models::Grapho;
use crate::models::graph_models::Node;

mod models;
const BASEPATH: &str = "./files";

fn main() {
    let mut runtime = Grapho::<String, Undirected>::new("test");
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
    println!("Graph runtime references: {:#?}", runtime.runtime_ref);
    runtime
        .save_to_file(&format!("{}/{}.json", BASEPATH, runtime.name))
        .unwrap()
}
