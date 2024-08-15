use crate::{config::logic_gate_config::LogicGateConfig, serialize_point::SerializablePoint};
use serde::{Deserialize, Serialize};

use super::node::Nodes;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GateType {
    And,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicGate {
    pub gate_type: GateType,
    pub position: SerializablePoint,
    pub width: f32,
    pub height: f32,
    pub nodes: Nodes,
}

impl LogicGate {
    pub fn new(
        gate_type: GateType,
        position: SerializablePoint,
        num_input_nodes: usize,
        num_output_nodes: usize,
        config: &LogicGateConfig,
    ) -> Self {
        let (width, height) = config.calculate_gate_size(num_input_nodes, num_output_nodes);
        let nodes = Nodes::new(num_input_nodes, num_output_nodes, position, height, width);

        Self {
            gate_type,
            position,
            width,
            height,
            nodes,
        }
    }

    pub fn update_output(&mut self) {
        let output = match self.gate_type {
            GateType::And => self.nodes.input_nodes[0].state && self.nodes.input_nodes[1].state,
            GateType::Not => !self.nodes.input_nodes[0].state,
        };

        self.nodes.output_nodes[0].state = output;
    }

    pub fn set_input(&mut self, index: usize, state: bool) {
        if index < self.nodes.input_nodes.len() {
            self.nodes.input_nodes[index].state = state;
            self.update_output();
        }
    }

    pub fn get_output(&self) -> bool {
        self.nodes.output_nodes[0].state
    }
}
