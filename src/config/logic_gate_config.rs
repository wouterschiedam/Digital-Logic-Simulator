use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicGateConfig {
    pub base_width: f32,
    pub node_height_spacing: f32,
    pub margin: f32,
    pub line_width: f32,
    pub node_radius: f32,
    pub small_node_radius: f32,
}

impl LogicGateConfig {
    pub fn new_default() -> Self {
        LogicGateConfig {
            base_width: 50.0,
            node_height_spacing: 15.0,
            margin: 10.0,
            line_width: 2.0,
            node_radius: 10.0,
            small_node_radius: 5.0,
        }
    }

    /// Dynamically calculate the gate size based on the number of input and output nodes
    pub fn calculate_gate_size(
        &self,
        num_input_nodes: usize,
        num_output_nodes: usize,
    ) -> (f32, f32) {
        let height =
            (num_input_nodes.max(num_output_nodes) as f32 + 1.0) * self.node_height_spacing;
        (self.base_width, height)
    }
}
