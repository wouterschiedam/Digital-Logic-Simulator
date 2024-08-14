use std::cell::{Cell, RefCell};

use iced::{
    executor,
    widget::{Button, Canvas, Column, Text},
    Application, Command, Element, Length, Point, Rectangle, Settings, Size, Theme,
};

use crate::{
    gates::{GateType, LogicGate, LogicGateApp},
    serialize_point::SerializablePoint,
};

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    Load,
    AddInputNode(Point),
    AddOutputNode(Point, Rectangle),
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
        let initial_gate = LogicGate {
            gate_type: GateType::Input,
            position: SerializablePoint { x: 25.0, y: 100.0 },
            inputs: Vec::new(),
            outputs: Vec::new(),
        };

        (
            Self {
                gates: vec![initial_gate].into(), // Initialize with one gate
                connections: RefCell::new(Vec::new()),
                current_drag_position: RefCell::new(None),
                current_dragging_line: RefCell::new(None),
                drag_start: RefCell::new(None),
                dragging_node: RefCell::new(None),
                is_dragging: Cell::new(false),
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
                    x: position.x,
                    y: position.y,
                };

                // Borrow the gates vector mutably from RefCell
                let mut gates = self.gates.borrow_mut();

                // Get the first mutable element (if any) and add the input node
                if let Some(gate) = gates.first_mut() {
                    gate.add_input_node(cursor_point);
                }
            }
            Message::AddOutputNode(position, bounds) => {
                let cursor_point = SerializablePoint {
                    x: position.x,
                    y: position.y,
                };

                // Borrow the gates vector mutably from RefCell
                let mut gates = self.gates.borrow_mut();

                if let Some(gate) = gates.first_mut() {
                    gate.add_output_node(cursor_point, bounds);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let save_button = Button::new(Text::new("Save")).on_press(Message::Save);
        let load_button = Button::new(Text::new("Load")).on_press(Message::Load);

        Column::new()
            .push(Canvas::new(self).width(Length::Fill).height(Length::Fill))
            .push(save_button)
            .push(load_button)
            .into()
    }

    fn title(&self) -> String {
        String::from("Logic Gate Simulator")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
