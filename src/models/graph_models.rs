use crate::models::node_types::NodeTypesProps;
use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::{EdgeType, Graph, graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

// -----------RUNTIME-REF---------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeRef {
    // Instances in petgraph core
    pub edges: Vec<Ref<EdgeIndex>>, // edge instance catalog (by core index)
    pub nodes: Vec<Ref<NodeIndex>>, // node catalog (by core index)
    // Reusable edge kinds catalog
    pub edge_kinds: Vec<Ref<Uuid>>, // reference to edge kind id by name
}
// -----------RUNTIME---------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct Grapho<T: NodeTypesProps, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub edges: Vec<Ref<EdgeIndex>>, // edge instances (core edges)
    pub runtime_ref: RuntimeRef,
    pub metadata: Metadata,
    pub edge_kinds: Vec<EdgeKind>, // reusable kinds (e.g., friendship)
}

impl<T: NodeTypesProps, Ty: EdgeType> Grapho<T, Ty>
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
                edge_kinds: vec![],
            },
            metadata: Metadata::new(description),
            edge_kinds: vec![],
        }
    }

    pub fn add_connection(
        &mut self,
        from: Node<T>,
        to: Node<T>,
        name: &str,
        description: &str,
    ) -> Result<(), String> {
        let from_idx = self.get_or_add_node(&from);
        let to_idx = self.get_or_add_node(&to);

        // Check if this exact connection (between from_idx and to_idx) with same edge kind name already exists
        let existing_edge_ref: Option<Ref<Uuid>> = self.core[from_idx]
            .connections
            .iter()
            .find(|c| c.edge.name == name && c.node.index == to_idx)
            .map(|c| c.edge.clone())
            .or_else(|| {
                self.core[to_idx]
                    .connections
                    .iter()
                    .find(|c| c.edge.name == name && c.node.index == from_idx)
                    .map(|c| c.edge.clone())
            });
        let already_exists = existing_edge_ref.is_some();

        if already_exists {
            let from_uuid = self.core[from_idx].metadata.id;
            let to_uuid = self.core[to_idx].metadata.id;
            let edge_ref: Ref<Uuid> = existing_edge_ref.unwrap();

            if !self.core[from_idx]
                .connections
                .iter()
                .any(|r| r.node.index == to_idx)
            {
                self.core[from_idx].connections.push(Connection {
                    node: Ref {
                        name: to.name.clone(),
                        uuid: to_uuid,
                        index: to_idx,
                    },
                    edge: edge_ref.clone(),
                });
            }
            if !self.core[to_idx]
                .connections
                .iter()
                .any(|r| r.node.index == from_idx)
            {
                self.core[to_idx].connections.push(Connection {
                    node: Ref {
                        name: from.name.clone(),
                        uuid: from_uuid,
                        index: from_idx,
                    },
                    edge: edge_ref.clone(),
                });
            }

            self.core[from_idx].node_index = from_idx;
            self.core[to_idx].node_index = to_idx;
            self.core[from_idx].energy = self.core[from_idx].connections.len() as u32;
            self.core[to_idx].energy = self.core[to_idx].connections.len() as u32;

            return Ok(());
        }

        let edge_index = self.core.add_edge(from_idx, to_idx, 1);
        // Find or create edge kind by name
        let edge_kind_id = if let Some(kind) = self.edge_kinds.iter().find(|k| k.name == name) {
            kind.metadata.id
        } else {
            let kind = EdgeKind::new(name, description);
            let id = kind.metadata.id;
            self.edge_kinds.push(kind);
            id
        };
        // Increment energy on edge kind for each new connection
        if let Some(kind) = self
            .edge_kinds
            .iter_mut()
            .find(|k| k.metadata.id == edge_kind_id)
        {
            kind.energy = kind.energy.saturating_add(1);
        }
        // Track runtime ref of this specific edge instance in the petgraph core
        if !self.runtime_ref.edges.iter().any(|r| r.index == edge_index) {
            self.runtime_ref.edges.push(Ref {
                name: name.to_string(),
                uuid: Uuid::new_v4(),
                index: edge_index,
            });
        }
        // Track runtime ref for edge kind
        if !self
            .runtime_ref
            .edge_kinds
            .iter()
            .any(|r| r.uuid == edge_kind_id)
        {
            self.runtime_ref.edge_kinds.push(Ref {
                name: name.to_string(),
                uuid: edge_kind_id,
                index: edge_kind_id,
            });
        }

        let from_uuid = self.core[from_idx].metadata.id;
        let to_uuid = self.core[to_idx].metadata.id;

        if !self.core[from_idx]
            .connections
            .iter()
            .any(|r| r.node.index == to_idx)
        {
            self.core[from_idx].connections.push(Connection {
                node: Ref {
                    name: to.name.clone(),
                    uuid: to_uuid,
                    index: to_idx,
                },
                edge: Ref {
                    name: name.to_string(),
                    uuid: edge_kind_id,
                    index: edge_kind_id,
                },
            });
        }
        if !self.core[to_idx]
            .connections
            .iter()
            .any(|r| r.node.index == from_idx)
        {
            self.core[to_idx].connections.push(Connection {
                node: Ref {
                    name: from.name.clone(),
                    uuid: from_uuid,
                    index: from_idx,
                },
                edge: Ref {
                    name: name.to_string(),
                    uuid: edge_kind_id,
                    index: edge_kind_id,
                },
            });
        }

        self.core[from_idx].node_index = from_idx;
        self.core[to_idx].node_index = to_idx;
        self.core[from_idx].energy = self.core[from_idx].connections.len() as u32;
        self.core[to_idx].energy = self.core[to_idx].connections.len() as u32;

        // Keep runtime nodes sorted by descending energy (relevance)
        self.runtime_ref
            .nodes
            .sort_by(|a, b| self.core[b.index].energy.cmp(&self.core[a.index].energy));

        Ok(())
    }

    fn get_or_add_node(&mut self, node: &Node<T>) -> NodeIndex {
        if let Some(existing_idx) = self
            .core
            .node_indices()
            .find(|&idx| self.core[idx].name == node.name)
        {
            self.core[existing_idx].node_index = existing_idx;
            return existing_idx;
        }
        let idx = self.core.add_node(node.clone());
        self.core[idx].node_index = idx;
        self.core[idx].energy = self.core[idx].connections.len() as u32;
        self.runtime_ref.nodes.push(Ref {
            name: node.name.clone(),
            uuid: node.metadata.id,
            index: idx,
        });
        idx
    }

    pub fn calculate_distance(&self, from: NodeIndex, to: NodeIndex) -> Result<u32, String> {
        let distance = petgraph::algo::dijkstra(&self.core, from, Some(to), |e| *e.weight());
        distance
            .get(&to)
            .cloned()
            .ok_or("No path found")
            .map_err(|e| format!("Error to calculate distance: {}", e))
    }

    pub fn get_node_index_by_name(&self, name: &str) -> Option<NodeIndex> {
        self.core
            .node_indices()
            .find(|&idx| self.core[idx].name == name)
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| format!("Error to read file: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Error to deserialize Graph: {}", e))
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

// -----------CONNECTIONS--------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub node: Ref<NodeIndex>,
    // Reference to an edge kind (shared across many connections)
    pub edge: Ref<Uuid>,
}
// -----------REFS---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ref<I> {
    pub uuid: Uuid,
    pub name: String,
    pub index: I,
}

// -----------NODE---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<T> {
    pub name: String,
    pub content: T,
    pub energy: u32,
    pub connections: Vec<Connection>,
    pub node_index: NodeIndex,
    pub metadata: Metadata,
}

impl<T> Node<T> {
    pub fn new(name: &str, content: T, connections: Vec<Connection>, description: &str) -> Self {
        Self {
            name: String::from(name),
            content,
            energy: connections.len() as u32,
            connections,
            node_index: NodeIndex::new(0),
            metadata: Metadata::new(description.to_string()),
        }
    }
}

// -----------EDGE KIND (reusable)---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeKind {
    pub name: String,
    pub energy: u32,
    pub metadata: Metadata,
}

impl EdgeKind {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            energy: 0,
            metadata: Metadata::new(description.to_string()),
        }
    }
}

// -----------METADATA---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: Uuid,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
