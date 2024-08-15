use crate::{
    components::{
        connection::Connection,
        gate::{GateType, LogicGate},
        line_path::LinePath,
        node::{Node, NodeType, Nodes},
    },
    config::logic_gate_config::LogicGateConfig,
    helpers::{
        self,
        helpers::{is_point_near_gate, is_point_near_node, NODE_RADIUS},
    },
    serialize_point::SerializablePoint,
};
use iced::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LogicGateAppState {
    pub nodes: Vec<Nodes>,
    pub gates: Vec<LogicGate>,
    pub connections: Vec<Connection>,
}

impl LogicGateAppState {
    pub fn new() -> Self {
        let input_node = Node::new(
            SerializablePoint { x: 25.0, y: 400.0 },
            NodeType::Input,
            NODE_RADIUS,
        );
        let output_node = Node::new(
            SerializablePoint {
                x: 1415.0,
                y: 400.0,
            },
            NodeType::Output,
            NODE_RADIUS,
        );

        let initial_nodes = Nodes {
            input_nodes: vec![input_node],
            output_nodes: vec![output_node],
        };

        Self {
            nodes: vec![initial_nodes],
            gates: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_gate(
        &mut self,
        gate_type: GateType,
        position: SerializablePoint,
        num_input_nodes: usize,
        num_output_nodes: usize,
        config: &LogicGateConfig,
    ) {
        let gate = LogicGate::new(
            gate_type,
            position,
            num_input_nodes,
            num_output_nodes,
            config,
        );
        self.gates.push(gate);
    }

    pub fn update_connections(&mut self) {
        let mut updates = Vec::new();
        for (node_index, node) in self.nodes.iter().enumerate() {
            if let Some(output) = node.input_nodes.get(0) {
                if output.state {
                    for connection in self.connections.iter_mut() {
                        if connection.from == node_index {
                            updates.push((connection.to, true));
                            connection.is_active = true;
                        }
                    }
                } else {
                    for connection in self.connections.iter_mut() {
                        if connection.from == node_index {
                            updates.push((connection.to, false));
                            connection.is_active = false;
                        }
                    }
                }
            }
        }

        for (node_index, state) in updates {
            if let Some(input) = self.nodes[node_index].input_nodes.get_mut(0) {
                input.state = state;
            }
            if let Some(output) = self.nodes[node_index].output_nodes.get_mut(0) {
                output.state = state;
            }

            for gate in self.gates.iter_mut() {
                if let Some(gate_input_node) = gate.nodes.input_nodes.get(0) {
                    if gate_input_node.position == self.nodes[node_index].input_nodes[0].position {
                        if gate.gate_type == GateType::Not {
                            if let Some(output_node) = gate.nodes.output_nodes.get_mut(0) {
                                output_node.state = !state;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn find_node_at_position(&self, position: Point) -> Option<(usize, NodeType)> {
        for node in self.nodes.iter() {
            for (node_index, input) in node.input_nodes.iter().enumerate() {
                if is_point_near_node(position, input) {
                    return Some((node_index, NodeType::Input));
                }
            }
            for (node_index, output) in node.output_nodes.iter().enumerate() {
                if is_point_near_node(position, output) {
                    return Some((node_index, NodeType::Output));
                }
            }
        }
        None
    }

    pub fn find_gate_at_position(&self, position: Point) -> Option<(usize, LogicGate)> {
        for (gate_index, gate) in self.gates.iter().enumerate() {
            if is_point_near_gate(position, gate) {
                return Some((gate_index, gate.clone()));
            }
        }
        None
    }

    pub fn check_proximity_to_nodes(
        &self,
        cursor_position: Point,
        start_position: &SerializablePoint,
    ) -> Option<SerializablePoint> {
        for node in self.nodes.iter() {
            for node in &node.input_nodes {
                if node.position != *start_position && is_point_near_node(cursor_position, node) {
                    return Some(node.position.clone());
                }
            }

            for node in &node.output_nodes {
                if node.position != *start_position && is_point_near_node(cursor_position, node) {
                    return Some(node.position.clone());
                }
            }
        }

        for gate in self.gates.iter() {
            for input_node in &gate.nodes.input_nodes {
                if input_node.position != *start_position
                    && is_point_near_gate(cursor_position, gate)
                {
                    return Some(input_node.position.clone());
                }
            }

            for output_node in &gate.nodes.output_nodes {
                if output_node.position != *start_position
                    && is_point_near_gate(cursor_position, gate)
                {
                    return Some(output_node.position.clone());
                }
            }
        }

        None
    }
}
