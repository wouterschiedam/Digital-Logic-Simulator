use iced::{
    event::Status,
    mouse::{Button, Cursor, Event},
    widget::canvas::{self, Frame, Geometry, Program},
    Rectangle, Renderer, Theme,
};

use crate::{
    components::node::NodeType,
    serialize_point::SerializablePoint,
    state::logic_gate_app_state::LogicGateAppState,
    ui::draw::{canvas_frame, canvas_free_nodes, canvas_gates},
};

use super::{
    logic_gate_app::{LogicGateApp, Message},
    update_draw::create_node,
};

impl Program<Message, Theme, Renderer> for LogicGateApp {
    type State = LogicGateAppState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(Event::ButtonPressed(Button::Right)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {}
            }
            canvas::Event::Mouse(Event::ButtonPressed(Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((node_index, node_type)) =
                        self.state.find_node_at_position(cursor_position)
                    {
                        let mut drag_start = None;
                        match node_type {
                            NodeType::Input => {
                                drag_start =
                                    Some(self.state.nodes[0].input_nodes[node_index].position);
                            }
                            NodeType::Output => {
                                drag_start =
                                    Some(self.state.nodes[0].output_nodes[node_index].position);
                            }
                        }
                        return (
                            Status::Captured,
                            Some(Message::UpdateDraggingNode(
                                Some((node_index, node_type)),
                                drag_start,
                            )),
                        );
                    }

                    if let Some((gate_index, gate)) =
                        self.state.find_gate_at_position(cursor_position)
                    {
                        let offset = SerializablePoint {
                            x: cursor_position.x - gate.position.x,
                            y: cursor_position.y - gate.position.y,
                        };

                        return (
                            Status::Captured,
                            Some(Message::UpdateDraggingGate(gate_index, offset)),
                        );
                    }

                    // Try creating a new node on the mouse click position
                    let (status, message) = create_node(cursor_position, bounds);
                    if status == Status::Captured {
                        return (status, message);
                    }
                }
            }
            canvas::Event::Mouse(Event::CursorMoved { position }) => {
                if let Some(dragging_node) = &state.dragging_node {
                    if let Some(line_path) = &mut state.current_dragging_line {
                        line_path.add_point(SerializablePoint::new(position.x, position.y));
                    }
                    return (Status::Captured, None);
                }

                if let Some(index) = self.state.dragging_gate_index {
                    if let Some(offset) = self.state.drag_start {
                        return (
                            Status::Captured,
                            Some(Message::UpdateDraggingGatePosition(position, index, offset)),
                        );
                    }
                }
            }
            canvas::Event::Mouse(Event::ButtonReleased(Button::Left)) => {
                if let Some((node_index, node_type)) = &self.state.dragging_node {
                    if *node_type == NodeType::Input {
                        return (
                            Status::Captured,
                            Some(Message::UpdateNodeState(*node_index, node_type.clone())),
                        );
                    }
                }
                return (Status::Captured, Some(Message::DisabledDragging));
            }
            _ => {}
        }

        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(&renderer, bounds.size());

        canvas_frame(&mut frame, bounds);

        canvas_free_nodes(&mut frame, &self.state.nodes);

        canvas_gates(&mut frame, &self.state.gates);

        vec![frame.into_geometry()]
    }
}
