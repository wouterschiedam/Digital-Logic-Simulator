use iced::{
    executor,
    widget::{row, Button, Canvas, Column, Text},
    Alignment, Application, Command, Element, Length, Point, Rectangle, Settings, Size, Theme,
};
use std::fs;

use crate::{
    components::{
        gate::{GateType, LogicGate},
        node::{Node, NodeType},
    },
    config::logic_gate_config::LogicGateConfig,
    helpers::helpers::NODE_RADIUS,
    serialize_point::SerializablePoint,
    state::logic_gate_app_state::LogicGateAppState,
};

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    Load,
    AddInputNode(Point),
    AddOutputNode(Point, Rectangle),
    AddGate(GateType, usize, usize),
    UpdateDraggingGate(usize, SerializablePoint),
    UpdateDraggingNode(Option<(usize, NodeType)>, Option<SerializablePoint>),
    UpdateDraggingGatePosition(Point, usize, SerializablePoint),
    UpdateNodeState(usize, NodeType),
    DisabledDragging,
}

pub struct LogicGateApp {
    pub state: LogicGateAppState,
}

pub fn run() -> iced::Result {
    LogicGateApp::run(Settings {
        window: iced::window::Settings {
            size: Size::new(1440.0, 920.0),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

impl LogicGateApp {
    pub fn new() -> Self {
        Self {
            state: LogicGateAppState::new(),
        }
    }

    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string(&self.state)?;
        fs::write(file_name, serialized)?;
        Ok(())
    }

    fn load_from_file(file_name: &str) -> Result<LogicGateAppState, std::io::Error> {
        let data = fs::read_to_string(file_name)?;
        let deserialized = serde_json::from_str(&data)?;
        Ok(deserialized)
    }

    fn get_config(&self) -> LogicGateConfig {
        LogicGateConfig::new_default()
    }
    pub fn update_position(&mut self, position: Point, index: usize, offset: SerializablePoint) {
        self.state.gates[index].position.x = position.x - offset.x;
        self.state.gates[index].position.y = position.y - offset.y;

        // Calculate the number of input and output nodes immutably first
        let num_input_nodes = self.state.gates[index].nodes.input_nodes.len();
        let num_output_nodes = self.state.gates[index].nodes.output_nodes.len();

        // Update the positions of the nodes relative to the new gate position
        let gate_width = self.state.gates[index].width;
        let gate_height = self.state.gates[index].height;
        let gate_position = self.state.gates[index].position.clone();

        // Update input nodes
        for (i, input_node) in self.state.gates[index]
            .nodes
            .input_nodes
            .iter_mut()
            .enumerate()
        {
            let y_position =
                gate_position.y + (i as f32 + 1.0) * gate_height / (num_input_nodes as f32 + 1.0);
            input_node.position.x = gate_position.x;
            input_node.position.y = y_position;
        }

        // Update output nodes
        for (i, output_node) in self.state.gates[index]
            .nodes
            .output_nodes
            .iter_mut()
            .enumerate()
        {
            let y_position =
                gate_position.y + (i as f32 + 1.0) * gate_height / (num_output_nodes as f32 + 1.0);
            output_node.position.x = gate_position.x + gate_width;
            output_node.position.y = y_position;
        }
    }
}

impl Application for LogicGateApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (LogicGateApp::new(), Command::none())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Save => {
                if let Err(e) = self.save_to_file("logic_gates.json") {
                    eprintln!("Failed to save: {}", e);
                }
            }
            Message::Load => match LogicGateApp::load_from_file("logic_gates.json") {
                Ok(loaded_state) => self.state = loaded_state,
                Err(e) => eprintln!("Failed to load: {}", e),
            },
            Message::AddInputNode(position) => {
                let cursor_point = SerializablePoint {
                    x: NODE_RADIUS + 10.0,
                    y: position.y,
                };

                let new_node = Node::new(cursor_point, NodeType::Input, NODE_RADIUS);

                if let Some(node) = self.state.nodes.first_mut() {
                    node.add_input_node(new_node);
                }
            }
            Message::AddOutputNode(position, bounds) => {
                let cursor_point = SerializablePoint {
                    x: bounds.width - NODE_RADIUS - 10.0,
                    y: position.y,
                };

                let new_node = Node::new(cursor_point, NodeType::Output, NODE_RADIUS);

                if let Some(node) = self.state.nodes.first_mut() {
                    node.add_output_node(new_node);
                }
            }
            Message::UpdateDraggingNode(node, start) => {
                self.state.dragging_node = node;
                self.state.is_dragging = true;
                self.state.drag_start = start;
            }
            Message::UpdateDraggingGate(gate_index, offset) => {
                self.state.dragging_gate_index = Some(gate_index);
                self.state.is_dragging = true;
                self.state.drag_start = Some(offset);
            }
            Message::UpdateDraggingGatePosition(position, index, offset) => {
                self.update_position(position, index, offset)
            }
            Message::DisabledDragging => {
                self.state.dragging_node = None;
                self.state.dragging_gate_index = None;
                self.state.is_dragging = false;
                self.state.drag_start = None;
            }
            Message::UpdateNodeState(node, node_type) => match node_type {
                NodeType::Input => {
                    self.state.nodes[0].input_nodes[node].state =
                        !self.state.nodes[0].input_nodes[node].state;
                }
                NodeType::Output => {
                    self.state.nodes[0].output_nodes[node].state =
                        !self.state.nodes[0].output_nodes[node].state;
                }
            },
            Message::AddGate(gate, input, output) => {
                self.state.add_gate(
                    gate,
                    SerializablePoint { x: 150.0, y: 400.0 },
                    input,
                    output,
                    &self.get_config(),
                );
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let save_button = Button::new(Text::new("Save")).on_press(Message::Save);
        let load_button = Button::new(Text::new("Load")).on_press(Message::Load);
        let not_gate =
            Button::new(Text::new("Not gate")).on_press(Message::AddGate(GateType::Not, 1, 1));

        Column::new()
            .push(Canvas::new(self).width(Length::Fill).height(Length::Fill))
            .push(
                Column::new()
                    .push(save_button)
                    .push(load_button)
                    .push(not_gate)
                    .spacing(10)
                    .align_items(Alignment::Center),
            )
            .into()
    }

    fn title(&self) -> String {
        String::from("Logic Gate Simulator")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
