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
pub struct Runtime<T, Ty: EdgeType> {
    pub name: String,
    pub core: Graph<Node<T>, u32, Ty>,
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
    pub runtime_ref: RuntimeRef,
    pub metadata: Metadata,
}

impl<T, Ty: EdgeType> Runtime<T, Ty>
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
            nodes: vec![],
            edges: vec![],
            runtime_ref: RuntimeRef {
                edges: vec![],
                nodes: vec![],
            },
            metadata: Metadata::new(description.to_string()),
        }
    }

    pub fn add_connection(
        &mut self,
        mut from: Node<T>,
        mut to: Node<T>,
        name: &str,
        description: &str,
    ) -> Result<(), String>
    where
        T: Clone,
    {
        // ---------- REUTILIZA OU ADICIONA NÓS ----------
        // FROM
        let from_idx = if let Some(existing_ref) = self
            .runtime_ref
            .nodes
            .iter()
            .find(|r| r.uuid == from.metadata.id)
        {
            existing_ref.index
        } else {
            let idx = self.core.add_node(from.clone());
            from.node_index = Some(idx);
            self.runtime_ref.nodes.push(Ref {
                uuid: from.metadata.id,
                index: idx,
            });
            self.nodes.push(from.clone());
            idx
        };

        // TO
        let to_idx = if let Some(existing_ref) = self
            .runtime_ref
            .nodes
            .iter()
            .find(|r| r.uuid == to.metadata.id)
        {
            existing_ref.index
        } else {
            let idx = self.core.add_node(to.clone());
            to.node_index = Some(idx);
            self.runtime_ref.nodes.push(Ref {
                uuid: to.metadata.id,
                index: idx,
            });
            self.nodes.push(to.clone());
            idx
        };

        // ---------- CRIA CONEXÃO ----------
        from.connections.push(Ref {
            uuid: from.metadata.id,
            index: from_idx,
        });
        to.connections.push(Ref {
            uuid: to.metadata.id,
            index: to_idx,
        });

        let edge_index = self.core.add_edge(from_idx, to_idx, 1);

        let edge = Edge::new(name, from_idx, to_idx, 1, edge_index, description);

        // Evita duplicação de referências de aresta
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

        // Atualiza metadados dos nós
        if let Some(node) = self
            .nodes
            .iter_mut()
            .find(|n| n.metadata.id == from.metadata.id)
        {
            node.metadata.update();
        }
        if let Some(node) = self
            .nodes
            .iter_mut()
            .find(|n| n.metadata.id == to.metadata.id)
        {
            node.metadata.update();
        }

        self.edges.push(edge);

        Ok(())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Erro ao serializar Runtime: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Erro ao escrever arquivo: {}", e))
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| format!("Erro ao ler arquivo: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Erro ao desserializar Runtime: {}", e))
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

    pub fn update(&mut self) {
        self.updated_at = Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
