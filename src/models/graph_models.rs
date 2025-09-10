use crate::models::node_types::NodeTypes;
use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::{EdgeType, Graph, graph::NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use uuid::Uuid;

// -----------RUNTIME-REF---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRef {
    // Instances in petgraph core
    pub edges: Vec<Ref<EdgeIndex>>, // edge instance catalog (by core index)
    pub nodes: Vec<Ref<NodeIndex>>, // node catalog (by core index)
    // Reusable edge kinds catalog
    pub edge_kinds: Vec<EdgeKindRef>, // reference to edge kind id by name
}
// -----------RUNTIME---------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct Grapho<T: NodeTypes, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub edges: Vec<Ref<EdgeIndex>>, // edge instances (core edges)
    pub runtime_ref: RuntimeRef,
    pub metadata: Metadata,
    pub edge_kinds: Vec<EdgeKind>, // reusable kinds (e.g., friendship)
    #[serde(skip)]
    node_index_by_name: HashMap<String, NodeIndex>,
}

impl<T: NodeTypes, Ty: EdgeType> Grapho<T, Ty>
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
            node_index_by_name: HashMap::new(),
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
        if let Some(edge_ref) = self.find_existing_connection(name, from_idx, to_idx) {
            self.ensure_connections_present(from_idx, to_idx, &from, &to, edge_ref);
            self.update_metrics_and_sort(from_idx, to_idx);
            return Ok(());
        }
        let edge_index = self.add_core_edge(from_idx, to_idx);
        let edge_kind_id = self.get_or_create_edge_kind(name, description);
        self.increment_edge_kind_energy(edge_kind_id);
        self.ensure_runtime_edge_instance(name, edge_index);
        self.ensure_runtime_edge_kind(name, edge_kind_id);
        self.push_connection(from_idx, to_idx, &from, &to, name, edge_kind_id);
        self.update_metrics_and_sort(from_idx, to_idx);
        Ok(())
    }

    fn find_existing_connection(
        &self,
        name: &str,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
    ) -> Option<Ref<Uuid>> {
        self.core[from_idx]
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
            })
    }

    fn ensure_connections_present(
        &mut self,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
        from: &Node<T>,
        to: &Node<T>,
        edge_ref: Ref<Uuid>,
    ) {
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
                edge: edge_ref,
            });
        }
    }

    fn add_core_edge(&mut self, from_idx: NodeIndex, to_idx: NodeIndex) -> EdgeIndex {
        self.core.add_edge(from_idx, to_idx, 1)
    }

    fn get_or_create_edge_kind(&mut self, name: &str, description: &str) -> Uuid {
        if let Some(kind) = self.edge_kinds.iter().find(|k| k.name == name) {
            return kind.metadata.id;
        }
        let kind = EdgeKind::new(name, description);
        let id = kind.metadata.id;
        self.edge_kinds.push(kind);
        id
    }

    fn increment_edge_kind_energy(&mut self, edge_kind_id: Uuid) {
        if let Some(kind) = self
            .edge_kinds
            .iter_mut()
            .find(|k| k.metadata.id == edge_kind_id)
        {
            kind.energy = kind.energy.saturating_add(1);
        }
    }

    fn ensure_runtime_edge_instance(&mut self, name: &str, edge_index: EdgeIndex) {
        if !self.runtime_ref.edges.iter().any(|r| r.index == edge_index) {
            self.runtime_ref.edges.push(Ref {
                name: name.to_string(),
                uuid: Uuid::new_v4(),
                index: edge_index,
            });
        }
    }

    fn ensure_runtime_edge_kind(&mut self, name: &str, edge_kind_id: Uuid) {
        if !self
            .runtime_ref
            .edge_kinds
            .iter()
            .any(|r| r.uuid == edge_kind_id)
        {
            self.runtime_ref.edge_kinds.push(EdgeKindRef {
                name: name.to_string(),
                uuid: edge_kind_id,
            });
        }
    }

    fn push_connection(
        &mut self,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
        from: &Node<T>,
        to: &Node<T>,
        name: &str,
        edge_kind_id: Uuid,
    ) {
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
    }

    fn update_metrics_and_sort(&mut self, from_idx: NodeIndex, to_idx: NodeIndex) {
        self.core[from_idx].node_index = from_idx;
        self.core[to_idx].node_index = to_idx;
        self.core[from_idx].energy = self.core[from_idx].connections.len() as u32;
        self.core[to_idx].energy = self.core[to_idx].connections.len() as u32;
        self.runtime_ref
            .nodes
            .sort_by(|a, b| self.core[b.index].energy.cmp(&self.core[a.index].energy));
    }

    fn get_or_add_node(&mut self, node: &Node<T>) -> NodeIndex {
        if let Some(idx) = self.node_index_by_name.get(&node.name).copied() {
            self.core[idx].node_index = idx;
            return idx;
        }
        let idx = self.core.add_node(node.clone());
        self.core[idx].node_index = idx;
        self.core[idx].energy = self.core[idx].connections.len() as u32;
        self.runtime_ref.nodes.push(Ref {
            name: node.name.clone(),
            uuid: node.metadata.id,
            index: idx,
        });
        self.node_index_by_name.insert(node.name.clone(), idx);
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
        self.node_index_by_name.get(name).copied()
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| format!("Error to read file: {}", e))?;
        let mut g: Self = serde_json::from_str(&data)
            .map_err(|e| format!("Error to deserialize Graph: {}", e))?;
        g.rebuild_indexes();
        Ok(g)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Error to create directories: {}", e))?;
        }
        let mut clone = self.clone_for_save();
        let json = serde_json::to_string_pretty(&clone)
            .map_err(|e| format!("Error to serialize Graph: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("Error to write file: {}", e))
    }

    pub fn save_to_file_encrypted(&self, path: &str, passphrase: &str) -> Result<(), String> {
        let mut clone = self.clone_for_save();
        crate::persist::save_encrypted(&clone, path, passphrase)
    }

    pub fn load_from_file_encrypted(path: &str, passphrase: &str) -> Result<Self, String> {
        let mut g: Self = crate::persist::load_encrypted(path, passphrase)?;
        g.rebuild_indexes();
        Ok(g)
    }

    fn rebuild_indexes(&mut self) {
        self.node_index_by_name.clear();
        for idx in self.core.node_indices() {
            let name = self.core[idx].name.clone();
            self.node_index_by_name.insert(name, idx);
        }
    }

    fn clone_for_save(&self) -> Self {
        let mut cloned = Self {
            name: self.name.clone(),
            core: self.core.clone(),
            edges: self.edges.clone(),
            runtime_ref: self.runtime_ref.clone(),
            metadata: self.metadata.clone(),
            edge_kinds: self.edge_kinds.clone(),
            node_index_by_name: HashMap::new(),
        };
        cloned.runtime_ref.nodes.sort_by_key(|r| r.index.index());
        cloned.runtime_ref.edges.sort_by_key(|r| r.index.index());
        cloned
            .runtime_ref
            .edge_kinds
            .sort_by(|a, b| a.name.cmp(&b.name));
        cloned.edge_kinds.sort_by(|a, b| a.name.cmp(&b.name));
        cloned
    }

    pub fn top_k_nodes_by_energy(&self, k: usize) -> Vec<Ref<NodeIndex>> {
        let mut v: Vec<_> = self.runtime_ref.nodes.iter().cloned().collect();
        v.sort_by(|a, b| self.core[b.index].energy.cmp(&self.core[a.index].energy));
        v.into_iter().take(k).collect()
    }

    pub fn neighbors_by_edge_kind(
        &self,
        node_idx: NodeIndex,
        edge_kind_name: &str,
    ) -> Vec<Ref<NodeIndex>> {
        self.core[node_idx]
            .connections
            .iter()
            .filter(|c| c.edge.name == edge_kind_name)
            .map(|c| c.node.clone())
            .collect()
    }

    pub fn edge_kind_stats(&self) -> Vec<(String, u32)> {
        let mut v: Vec<_> = self
            .edge_kinds
            .iter()
            .map(|k| (k.name.clone(), k.energy))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeKindRef {
    pub uuid: Uuid,
    pub name: String,
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
