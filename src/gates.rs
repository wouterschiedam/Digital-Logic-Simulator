use iced::Rectangle;
use serde::{Deserialize, Serialize};

use crate::{
    helpers::NODE_RADIUS,
    nodes::{Node, NodeType},
    serialize_point::SerializablePoint,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateType {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicGate {
    pub gate_type: GateType,
    pub position: SerializablePoint,
    pub input_nodes: Vec<Node>,
    pub output_nodes: Vec<Node>,
    pub inputs: Vec<bool>,
    pub outputs: Vec<bool>,
}

impl LogicGate {
    pub fn new(gate_type: GateType, position: SerializablePoint) -> Self {
        let inputs = match gate_type {
            GateType::Not => vec![false], // NOT gate has 1 input
            _ => vec![false, false],      // AND, OR gates have 2 inputs
        };
        let outputs = vec![false]; // All gates have 1 output
        Self {
            gate_type,
            position,
            input_nodes: Vec::new(),
            output_nodes: Vec::new(),
            inputs,
            outputs,
        }
    }

    pub fn add_input_node(&mut self, position: SerializablePoint) {
        let node_position = SerializablePoint {
            x: NODE_RADIUS + 10.0,
            y: position.y,
        };

        self.input_nodes.push(Node {
            position: node_position,
            state: false,
            connected_to: None,
            node_type: NodeType::Input,
        });
    }

    pub fn add_output_node(&mut self, position: SerializablePoint, bounds: Rectangle) {
        let node_position = SerializablePoint {
            x: bounds.width - NODE_RADIUS - 10.0,
            y: position.y,
        };

        self.output_nodes.push(Node {
            position: node_position,
            state: false,
            connected_to: None,
            node_type: NodeType::Output,
        });
    }

    // Compute the output based on the gate type and inputs
    pub fn update_output(&mut self) {
        let output = match self.gate_type {
            GateType::And => self.inputs[0] && self.inputs[1],
            GateType::Or => self.inputs[0] || self.inputs[1],
            GateType::Not => !self.inputs[0],
        };
        self.outputs[0] = output;
    }

    // Set the state of a specific input
    pub fn set_input(&mut self, index: usize, state: bool) {
        if index < self.inputs.len() {
            self.inputs[index] = state;
            self.update_output(); // Update output when input changes
        }
    }

    // Get the state of the output
    pub fn get_output(&self) -> bool {
        self.outputs[0]
    }
}



