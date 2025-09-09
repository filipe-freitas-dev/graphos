use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::{EdgeType, Graph, graph::NodeIndex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// -----------RUNTIME-REF---------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeRef {
    pub edges: Vec<Ref<EdgeIndex>>,
    pub nodes: Vec<Ref<NodeIndex>>,
}
// -----------RUNTIME---------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct Grapho<T, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub edges: Vec<Edge>,
    pub runtime_ref: RuntimeRef,
    pub metadata: Metadata,
}

impl<T, Ty: EdgeType> Grapho<T, Ty>
where
    T: Clone + Serialize + for<'de> Deserialize<'de>,
    Ty: EdgeType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(name: &str) -> Self {
        let core = Graph::<Node<T>, u32, Ty>::default();
        let description = format!("{} graph.", name);
        Self {
            name: name.to_string(),
            core,
            edges: vec![],
            runtime_ref: RuntimeRef {
                edges: vec![],
                nodes: vec![],
            },
            metadata: Metadata::new(description),
        }
    }

    pub fn add_connection(
        &mut self,
        from: Node<T>,
        to: Node<T>,
        name: &str,
        description: &str,
    ) -> Result<(), String> {
        let from_idx = self.get_or_add_node(from);
        let to_idx = self.get_or_add_node(to);

        // Cria relação no grafo e obtém índice da aresta
        let edge_index = self.core.add_edge(from_idx, to_idx, 1);

        // Cria aresta rica
        let edge = Edge::new(name, from_idx, to_idx, 1, edge_index, description);

        // Adiciona no vetor de arestas e nas referências
        if !self
            .runtime_ref
            .edges
            .iter()
            .any(|r| r.uuid == edge.metadata.id)
        {
            self.runtime_ref.edges.push(Ref {
                uuid: edge.metadata.id,
                index: edge_index,
            });
        }

        self.edges.push(edge);
        Ok(())
    }

    fn get_or_add_node(&mut self, node: Node<T>) -> NodeIndex {
        if let Some(existing_ref) = self
            .runtime_ref
            .nodes
            .iter()
            .find(|r| r.uuid == node.metadata.id)
        {
            existing_ref.index
        } else {
            let idx = self.core.add_node(node.clone());
            self.runtime_ref.nodes.push(Ref {
                uuid: node.metadata.id,
                index: idx,
            });
            idx
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Error to create directories: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Error to serialize Graph: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("Error to write file: {}", e))
    }
}

// -----------REFS---------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref<I> {
    pub uuid: Uuid,
    pub index: I,
}

// -----------NODE---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<T> {
    pub name: String,
    pub content: T,
    pub connections: Vec<Ref<NodeIndex>>,
    pub node_index: NodeIndex,
    pub metadata: Metadata,
}

impl<T> Node<T> {
    pub fn new(
        name: &str,
        content: T,
        connections: Vec<Ref<NodeIndex>>,
        description: &str,
    ) -> Self {
        Self {
            name: String::from(name),
            content,
            connections,
            node_index: NodeIndex::new(0),
            metadata: Metadata::new(description.to_string()),
        }
    }
}

// -----------EDGE---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub name: String,
    pub from: NodeIndex,
    pub to: NodeIndex,
    pub weight: u32,
    pub edge_index: EdgeIndex,
    pub metadata: Metadata,
}

impl Edge {
    pub fn new(
        name: &str,
        from: NodeIndex,
        to: NodeIndex,
        weight: u32,
        edge_index: EdgeIndex,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            from,
            to,
            weight,
            edge_index,
            metadata: Metadata::new(description.to_string()),
        }
    }
}

// -----------METADATA---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
}

impl Metadata {
    pub fn new(description: String) -> Self {
        let now = Utc::now();

        Self {
            created_at: now,
            updated_at: now,
            id: Uuid::new_v4(),
            description,
        }
    }

    // pub fn update(&mut self) {
    //     self.updated_at = Utc::now()
    // }
}
