use chrono::{DateTime, Utc};
use petgraph::prelude::EdgeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{EdgeType, Graph, adj::NodeIndex};
use std::collections::HashMap;
use uuid::Uuid;

// -----------GRAPHO---------------------------------
#[derive(Debug)]
pub struct Grapho<T, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
    pub node_indices: HashMap<Uuid, NodeIndex>,
    pub metadata: Metadata,
}

impl<T, Ty: EdgeType> Grapho<T, Ty> {
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

    pub fn add_connection(&mut self, from: Node<T>, to: Node<T>, name: &str) -> Result<(), String> {
        let edge_index = self.core.add_edge(
            from.node_index.unwrap().into(),
            to.node_index.unwrap().into(),
            1,
        );

        let edge = Edge::new(
            name,
            from.node_index.unwrap(),
            to.node_index.unwrap(),
            1,
            edge_index,
        );

        self.edges.push(edge);

        Ok(())
    }
}

// -----------NODE---------------------------------
#[derive(Debug)]
pub struct Node<T> {
    pub name: String,
    pub content: T,
    pub connections: Vec<Edge>,
    pub node_index: Option<NodeIndex>,
    pub metadata: Metadata,
}

impl<T> Node<T> {
    pub fn new(
        name: &str,
        content: T,
        connections: Vec<Edge>,
        description: &str,
        node_index: Option<NodeIndex>,
    ) -> Self {
        Self {
            name: String::from(name),
            content,
            connections,
            node_index,
            metadata: Metadata::new(String::from(description)),
        }
    }

    pub fn connect(&mut self, vertice_id: Node<T>, name: &str) -> Result<(), String> {
        Ok(())
    }
}

// -----------EDGE---------------------------------
#[derive(Debug)]
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
    ) -> Self {
        let description = format!("{} --> {}", from, to);

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
#[derive(Debug)]
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
