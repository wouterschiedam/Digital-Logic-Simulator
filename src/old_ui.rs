use std::cell::{Cell, RefCell};

use iced::{
    executor,
    widget::{row, Button, Canvas, Column, Text},
    Alignment, Application, Command, Element, Length, Point, Rectangle, Settings, Size, Theme,
};

use crate::{
    gates::{GateType, LogicGate},
    helpers::NODE_RADIUS,
    nodes::{Node, NodeType, Nodes},
    serialize_point::SerializablePoint,
    LGapp::LogicGateApp,
};

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    Load,
    AddInputNode(Point),
    AddOutputNode(Point, Rectangle),
    AddGate(GateType, usize, usize),
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

impl Application for LogicGateApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let initial_node = Nodes {
            input_nodes: Vec::new(),
            output_nodes: Vec::new(),
        };
        //let initial_gates = LogicGate::new(GateType::Not, SerializablePoint::new(0.0, 0.0));
        (
            Self {
                nodes: RefCell::new(vec![initial_node]),
                gates: RefCell::new(None),
                connections: RefCell::new(Vec::new()),
                current_drag_position: RefCell::new(None),
                current_dragging_line: RefCell::new(None),
                drag_start: RefCell::new(None),
                dragging_node: RefCell::new(None),
                is_dragging: Cell::new(false),
                dragging_gate_index: RefCell::new(None),
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Save => {
                if let Err(e) = self.save_to_file("logic_gates.json") {
                    eprintln!("Failed to save: {}", e);
                }
            }
            Message::Load => match LogicGateApp::load_from_file("logic_gates.json") {
                Ok(loaded_state) => *self = loaded_state,
                Err(e) => eprintln!("Failed to load: {}", e),
            },
            Message::AddInputNode(position) => {
                let cursor_point = SerializablePoint {
                    x: NODE_RADIUS + 10.0,
                    y: position.y,
                };

                let new_node = Node::new(cursor_point, NodeType::Output);

                // Borrow the gates vector mutably from RefCell
                let mut nodes = self.nodes.borrow_mut();

                // Get the first mutable element (if any) and add the input node
                if let Some(node) = nodes.first_mut() {
                    node.add_input_node(new_node);
                }
            }
            Message::AddOutputNode(position, bounds) => {
                let cursor_point = SerializablePoint {
                    x: bounds.width - NODE_RADIUS - 10.0,
                    y: position.y,
                };

                let new_node = Node::new(cursor_point, NodeType::Output);

                // Borrow the gates vector mutably from RefCell
                let mut nodes = self.nodes.borrow_mut();

                if let Some(node) = nodes.first_mut() {
                    node.add_output_node(new_node);
                }
            }
            Message::AddGate(gate, input, output) => {
                self.add_gate(
                    gate,
                    SerializablePoint { x: 150.0, y: 400.0 },
                    input,
                    output,
                );
            }
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
                row![save_button, load_button, not_gate]
                    .spacing(10)
                    .width(Length::Fill)
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
