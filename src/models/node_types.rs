use serde::{Deserialize, Serialize};

pub trait NodeTypes {} // market trait for node types (just to crate some "inheritance" to include the generics type for Grapho struct)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Text(String),
    Num(isize),
    Fnum32(f32),
    Fnum64(f64),
    Bool(bool),
    Seq(Vec<NodeType>),
}

impl NodeTypes for NodeType {}
