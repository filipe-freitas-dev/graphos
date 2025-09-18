use petgraph::EdgeType;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, path::PathBuf, time::{Duration, SystemTime}}; 

use crate::models::{graph_models::{Grapho, Metadata}, node_types::NodeTypes};

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
pub struct GraphMetadata {
    pub name: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub edge_kinds: Vec<String>,
    pub top_nodes: Vec<String>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct LoaderConfig {
    pub cache_ttl: Duration
    pub max_memory_usage: usize,
    pub prefetch_popular: bool,
    pub auto_cleanup: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(300),
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            prefetch_popular: true,
            auto_cleanup: true
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphLoaderAgent {
    config: LoaderConfig,
    file_registry: HashMap<String, CacheEntry>,
    metadata_cache: HashMap<String, GraphMetadata>,
    access_frequency: HashMap<String, u32>,
    memory_usage: usize,
}
impl GraphLoaderAgent {
    pub fn new(config: LoaderConfig) -> Self {
        Self {
            config,
            file_registry: HashMap::new(),
            metadata_cache: HashMap::new(),
            access_frequency: HashMap::new(),
            memory_usage: 0
        }
    }
}
