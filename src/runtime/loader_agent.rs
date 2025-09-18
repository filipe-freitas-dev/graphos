use petgraph::EdgeType;
use serde::{Serialize, Deserialize};
use std::{path::PathBuf, time::SystemTime}

use crate::models::{graph_models::Grapho, node_types::NodeTypes};

pub trait LoadGraphOperations<T: NodeTypes, Ty: EdgeType> {
    pub fn load_graph(&mut self, name: &str) -> Result<Grapho<T,Ty>, String>;
    pub fn peek_metadata(&mut self, name: &str ) -> Result<GraphMetadata, String>;
    pub fn is_available(&self, name: &str) -> bool;
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    file_path: PathBuf,
    last_accessed: SystemTime,
    last_modified: SystemTime,
    file_size: u64,
    is_encrypted: bool,
    passphrase: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata;
