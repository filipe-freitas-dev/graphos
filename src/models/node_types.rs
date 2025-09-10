use serde::{Deserialize, Serialize};

pub trait NodeTypesProps {} // market trait for node types (just to crate some "inheritance" to include the generics type for Grapho struct)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeTypes {
    Text(String),
    Num(isize),
    Fnum32(f32),
    Fnum64(f64),
    Bool(bool),
    Seq(Vec<NodeTypes>),
}

impl NodeTypesProps for NodeTypes {}
