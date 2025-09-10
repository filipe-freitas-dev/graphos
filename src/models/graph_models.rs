use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::{EdgeType, Graph, graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::fs;
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
    pub edges: Vec<Ref<EdgeIndex>>,
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

        let existing_edge_props: Option<EdgeProps> = self.core[from_idx]
            .connections
            .iter()
            .find(|c| {
                c.edge.name == name
                    && ((c.edge.from == from_idx && c.edge.to == to_idx)
                        || (c.edge.from == to_idx && c.edge.to == from_idx))
            })
            .map(|c| c.edge.clone())
            .or_else(|| {
                self.core[to_idx]
                    .connections
                    .iter()
                    .find(|c| {
                        c.edge.name == name
                            && ((c.edge.from == from_idx && c.edge.to == to_idx)
                                || (c.edge.from == to_idx && c.edge.to == from_idx))
                    })
                    .map(|c| c.edge.clone())
            });
        let already_exists = existing_edge_props.is_some();

        if already_exists {
            let from_uuid = self.core[from_idx].metadata.id;
            let to_uuid = self.core[to_idx].metadata.id;
            let edge_props: EdgeProps = existing_edge_props.unwrap();

            if !self.core[from_idx]
                .connections
                .iter()
                .any(|r| r.node.index == to_idx)
            {
                self.core[from_idx].connections.push(Connection {
                    node: Ref {
                        uuid: to_uuid,
                        index: to_idx,
                    },
                    edge: edge_props.clone(),
                });
            }
            if !self.core[to_idx]
                .connections
                .iter()
                .any(|r| r.node.index == from_idx)
            {
                self.core[to_idx].connections.push(Connection {
                    node: Ref {
                        uuid: from_uuid,
                        index: from_idx,
                    },
                    edge: edge_props.clone(),
                });
            }

            self.core[from_idx].node_index = from_idx;
            self.core[to_idx].node_index = to_idx;
            self.core[from_idx].energy = self.core[from_idx].connections.len() as u32;
            self.core[to_idx].energy = self.core[to_idx].connections.len() as u32;

            return Ok(());
        }

        let edge_index = self.core.add_edge(from_idx, to_idx, 1);

        let edge = EdgeProps::new(name, from_idx, to_idx, 1, edge_index, description);
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

        self.edges.push(Ref::<EdgeIndex> {
            uuid: edge.metadata.id,
            index: edge_index,
        });

        let from_uuid = self.core[from_idx].metadata.id;
        let to_uuid = self.core[to_idx].metadata.id;

        if !self.core[from_idx]
            .connections
            .iter()
            .any(|r| r.node.index == to_idx)
        {
            self.core[from_idx].connections.push(Connection {
                node: Ref {
                    uuid: to_uuid,
                    index: to_idx,
                },
                edge: edge.clone(),
            });
        }
        if !self.core[to_idx]
            .connections
            .iter()
            .any(|r| r.node.index == from_idx)
        {
            self.core[to_idx].connections.push(Connection {
                node: Ref {
                    uuid: from_uuid,
                    index: from_idx,
                },
                edge: edge,
            });
        }

        self.core[from_idx].node_index = from_idx;
        self.core[to_idx].node_index = to_idx;
        self.core[from_idx].energy = self.core[from_idx].connections.len() as u32;
        self.core[to_idx].energy = self.core[to_idx].connections.len() as u32;

        Ok(())
    }

    fn get_or_add_node(&mut self, node: Node<T>) -> NodeIndex {
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
    pub edge: EdgeProps,
}
// -----------REFS---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ref<I> {
    pub uuid: Uuid,
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

// -----------EDGE---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeProps {
    name: String,
    from: NodeIndex,
    to: NodeIndex,
    weight: u32,
    edge_index: EdgeIndex,
    metadata: Metadata,
}

impl EdgeProps {
    fn new(
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
