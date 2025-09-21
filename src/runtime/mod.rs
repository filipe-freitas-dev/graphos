use std::collections::HashMap;
use petgraph::EdgeType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::graph_models::Grapho;
use crate::models::node_types::NodeTypes;

pub mod persist;
pub mod loader_agent;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GraphSpace<T: NodeTypes, Ty: EdgeType> {
    graphs: HashMap<String, Grapho<T, Ty>>,
    global_edge_kinds: HashMap<String, Uuid>,
}

impl<T: NodeTypes + Clone + Serialize + for<'de> Deserialize<'de>, Ty: EdgeType + Serialize + for<'de> Deserialize<'de>> GraphSpace<T, Ty> {
    pub fn new() -> Self {
        Self { graphs: HashMap::new(), global_edge_kinds: HashMap::new() }
    }

    pub fn add_graph(&mut self, graph: Grapho<T, Ty>) {
        self.graphs.insert(graph.name.clone(), graph);
    }

    pub fn get_graph(&self, name: &str) -> Option<&Grapho<T, Ty>> {
        self.graphs.get(name)
    }

    pub fn get_graph_mut(&mut self, name: &str) -> Option<&mut Grapho<T, Ty>> {
        self.graphs.get_mut(name)
    }

    pub fn link_edge_kind(&mut self, name: &str) -> Uuid {
        if let Some(id) = self.global_edge_kinds.get(name) { return *id; }
        let id = Uuid::new_v4();
        self.global_edge_kinds.insert(name.to_string(), id);
        id
    }
}
