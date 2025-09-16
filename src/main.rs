use petgraph::Undirected;

use crate::models::graph_models::Grapho;
use crate::models::graph_models::Node;
use crate::models::node_types::NodeType;

mod models;
mod runtime;
const BASEPATH: &str = "./files";

fn main() {
    let mut grapho = Grapho::<NodeType, Undirected>::new("graphos");
    let _ = grapho.add_connection(
        Node::new(
            "person1",
            NodeType::Text("filipe".to_string()),
            "description of node1",
        ),
        Node::new(
            "person2",
            NodeType::Text("maria".to_string()),
            "description of node2",
        ),
        "friendship",
        "description of friendship",
    );
    let _ = grapho.add_connection(
        Node::new(
            "person2",
            NodeType::Text("maria".to_string()),
            "description of node2",
        ),
        Node::new(
            "person3",
            NodeType::Text("joao".to_string()),
            "description of node3",
        ),
        "friendship",
        "description of friendship",
    );
    let _ = grapho.add_connection(
        Node::new(
            "person2",
            NodeType::Text("maria".to_string()),
            "description of node2",
        ),
        Node::new(
            "person4",
            NodeType::Text("henrique".to_string()),
            "description of node4",
        ),
        "friendship",
        "description of friendship",
    );
    let full_path = format!("{}/{}.json", BASEPATH, grapho.name);
    grapho.save_to_file(&full_path).unwrap();
    let new_graph = Grapho::<NodeType, Undirected>::load_from_file(&full_path).unwrap();
    println!("New graph: {:#?}", new_graph);

    let enc_path = format!("{}/{}-enc.bin", BASEPATH, new_graph.name);
    let pass = "change-this-passphrase";
    new_graph.save_to_file_encrypted(&enc_path, pass).unwrap();
    let _new_graph_enc =
        Grapho::<NodeType, Undirected>::load_from_file_encrypted(&enc_path, pass).unwrap();

    let mut global = runtime::GraphRuntime::<NodeType, Undirected>::new();
    global.add_graph(new_graph);
}

