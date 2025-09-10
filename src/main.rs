use petgraph::Undirected;

use crate::models::graph_models::Grapho;
use crate::models::graph_models::Node;
use crate::models::node_types::NodeTypes;

mod models;
const BASEPATH: &str = "./files";

fn main() {
    let mut runtime = Grapho::<NodeTypes, Undirected>::new("graphos");
    let _ = runtime.add_connection(
        Node::new(
            "person1",
            NodeTypes::Text("filipe".to_string()),
            vec![],
            "description of node1",
        ),
        Node::new(
            "person2",
            NodeTypes::Text("maria".to_string()),
            vec![],
            "description of node2",
        ),
        "friendship",
        "description of friendship",
    );
    let _ = runtime.add_connection(
        Node::new(
            "person2",
            NodeTypes::Text("maria".to_string()),
            vec![],
            "description of node2",
        ),
        Node::new(
            "person3",
            NodeTypes::Text("joao".to_string()),
            vec![],
            "description of node3",
        ),
        "friendship",
        "description of friendship",
    );
    let _ = runtime.add_connection(
        Node::new(
            "person2",
            NodeTypes::Text("maria".to_string()),
            vec![],
            "description of node2",
        ),
        Node::new(
            "person4",
            NodeTypes::Text("henrique".to_string()),
            vec![],
            "description of node4",
        ),
        "friendship",
        "description of friendship",
    );
    println!("Graph edges: {:#?}", runtime.edges);
    println!("Core: {:#?}", runtime.core);
    println!("Graph runtime references: {:#?}", runtime.runtime_ref);
    if let (Some(a), Some(b)) = (
        runtime.get_node_index_by_name("person4"),
        runtime.get_node_index_by_name("person3"),
    ) {
        println!(
            "Graph distance: {:#?}",
            runtime.calculate_distance(a, b).unwrap()
        );
    } else {
        println!("Graph distance: could not find required nodes");
    }
    let full_path = format!("{}/{}.json", BASEPATH, runtime.name);
    runtime.save_to_file(&full_path).unwrap();
    let new_graph = Grapho::<NodeTypes, Undirected>::load_from_file(&full_path).unwrap();
    println!("New graph: {:#?}", new_graph);
}
