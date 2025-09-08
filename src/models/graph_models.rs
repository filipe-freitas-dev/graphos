use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::{EdgeType, Graph, graph::NodeIndex};
use std::collections::HashMap;
use uuid::Uuid;

// -----------RUNTIME-REF---------------------------------
#[derive(Debug)]
pub struct RuntimeRef {
    pub edges: Vec<Ref<EdgeIndex>>,
    pub nodes: Vec<Ref<NodeIndex>>,
}
// -----------RUNTIME---------------------------------
#[derive(Debug)]
pub struct Runtime<T, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
    pub runtime_ref: RuntimeRef,
    pub node_indices: HashMap<Uuid, NodeIndex>,
    pub metadata: Metadata,
}

impl<T, Ty: EdgeType> Runtime<T, Ty> {
    pub fn new(name: &str) -> Self {
        let core = Graph::<Node<T>, u32, Ty>::default();
        let description = format!("{} graph.", name);
        Self {
            name: name.to_string(),
            core,
            nodes: vec![],
            edges: vec![],
            runtime_ref: RuntimeRef {
                edges: vec![],
                nodes: vec![],
            },
            node_indices: HashMap::new(),
            metadata: Metadata::new(description.to_string()),
        }
    }

    pub fn add_connection(
        &mut self,
        from: Node<T>,
        to: Node<T>,
        name: &str,
        description: &str,
    ) -> Result<(), String>
    where
        T: Clone,
    {
        let mut from_clone = from.clone();
        let mut to_clone = to.clone();
        let from_idx = self.core.add_node(from);
        let to_idx = self.core.add_node(to);
        from_clone.node_index = Some(from_idx);
        to_clone.node_index = Some(to_idx);
        from_clone.connections.push(Ref {
            uuid: from_clone.metadata.id,
            index: from_idx,
        });
        to_clone.connections.push(Ref {
            uuid: to_clone.metadata.id,
            index: to_idx,
        });
        self.runtime_ref.nodes.push(Ref {
            uuid: from_clone.metadata.id,
            index: from_idx,
        });

        self.runtime_ref.nodes.push(Ref {
            uuid: to_clone.metadata.id,
            index: to_idx,
        });

        let edge_index = self.core.add_edge(from_idx, to_idx, 1);

        let edge = Edge::new(name, from_idx, to_idx, 1, edge_index, description);
        self.runtime_ref.edges.push(Ref {
            uuid: edge.metadata.id,
            index: edge_index,
        });

        self.nodes.push(from_clone);
        self.nodes.push(to_clone);
        self.edges.push(edge);

        Ok(())
    }
}

// -----------REFS---------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Ref<I> {
    pub uuid: Uuid,
    pub index: I,
}

// -----------NODE---------------------------------
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub name: String,
    pub content: T,
    pub connections: Vec<Ref<NodeIndex>>,
    pub node_index: Option<NodeIndex>,
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
            node_index: None,
            metadata: Metadata::new(description.to_string()),
        }
    }
}

// -----------EDGE---------------------------------
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

    pub fn update(&mut self) {
        self.updated_at = Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
