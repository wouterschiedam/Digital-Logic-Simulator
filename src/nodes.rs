use serde::{Deserialize, Serialize};

use crate::serialize_point::SerializablePoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub position: SerializablePoint,
    pub state: bool,
    pub connected_to: Option<usize>, // Index of the connected node (if any)
    pub node_type: NodeType,
}

impl Node {
    pub fn new(position: SerializablePoint, node_type: NodeType) -> Self {
        Self {
            position,
            state: false,
            connected_to: None,
            node_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nodes {
    pub input_nodes: Vec<Node>,
    pub output_nodes: Vec<Node>,
}

impl Nodes {
    pub fn new(input: usize, output: usize, gate_position: SerializablePoint) -> Self {
        Self {
            input_nodes: Self::create_input_nodes(input, &gate_position),
            output_nodes: Self::create_output_nodes(output, &gate_position),
        }
    }

    pub fn create_input_nodes(count: usize, gate_position: &SerializablePoint) -> Vec<Node> {
        (0..count)
            .map(|_i| {
                Node::new(
                    SerializablePoint::new(gate_position.x, gate_position.y),
                    NodeType::Input,
                )
            })
            .collect()
    }

    pub fn create_output_nodes(count: usize, gate_position: &SerializablePoint) -> Vec<Node> {
        (0..count)
            .map(|_i| {
                Node::new(
                    SerializablePoint::new(gate_position.x, gate_position.y),
                    NodeType::Output,
                )
            })
            .collect()
    }

    pub fn add_input_node(&mut self, node: Node) {
        self.input_nodes.push(node);
    }

    pub fn add_output_node(&mut self, node: Node) {
        self.output_nodes.push(node);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Input,
    Output,
}
