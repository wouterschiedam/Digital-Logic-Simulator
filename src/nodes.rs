use serde::{Deserialize, Serialize};

use crate::serialize_point::SerializablePoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub position: SerializablePoint,
    pub state: bool,
    pub connected_to: Option<usize>, // Index of the connected node (if any)
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Input,
    Output,
}
