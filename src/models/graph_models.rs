use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::{EdgeType, Graph, graph::NodeIndex};
use std::collections::HashMap;
use uuid::Uuid;

// -----------RUNTIME---------------------------------
#[derive(Debug)]
pub struct Runtime<T, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
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
        self.nodes.push(from.clone());
        self.nodes.push(to.clone());
        // Add the nodes to the graph; capture their actual indices assigned by petgraph
        let from_idx = self.core.add_node(from);
        let to_idx = self.core.add_node(to);

        // Now add the edge using the valid indices
        let edge_index = self.core.add_edge(from_idx, to_idx, 1);

        let edge = Edge::new(name, from_idx, to_idx, 1, edge_index, description);

        self.edges.push(edge);

        Ok(())
    }
}

// -----------NODE---------------------------------
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub name: String,
    pub content: T,
    pub connections: Vec<Edge>,
    pub node_index: NodeIndex,
    pub metadata: Metadata,
}

impl<T> Node<T> {
    pub fn new(
        name: &str,
        content: T,
        connections: Vec<Edge>,
        node_index: Option<NodeIndex>,
        description: &str,
    ) -> Self {
        let node_index = match node_index {
            Some(node_index) => node_index,
            None => NodeIndex::new(0),
        };
        Self {
            name: String::from(name),
            content,
            connections,
            node_index,
            metadata: Metadata::new(description.to_string()),
        }
    }

    pub fn connect(&mut self, vertice_id: Node<T>, name: &str) -> Result<(), String> {
        Ok(())
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

    // pub fn add_to_grapho<T, Ty: petgraph::EdgeType>(
    //     &self,
    //     grapho: &mut Grapho<T, Ty>,
    // ) -> Result<(), String> {
    //     grapho
    //         .core
    //         .add_edge(self.from.into(), self.to.into(), self.weight);

    //     grapho.edges.push(self.metadata.id);

    //     Ok(())
    // }
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
