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
    ui::draw::{
        canvas_connection_on_the_fly, canvas_connections, canvas_frame, canvas_free_nodes,
        canvas_gates,
    },
};

use super::{
    logic_gate_app::{LogicGateApp, Message},
    update_draw::create_node,
};

impl Program<Message, Theme, Renderer> for LogicGateApp {
    type State = LogicGateAppState;

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(Event::ButtonPressed(Button::Right)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((node_index, node_type)) =
                        self.state.find_node_at_position(cursor_position)
                    {
                        return (
                            Status::Captured,
                            Some(Message::RemoveNode(node_index, node_type)),
                        );
                    }
                }
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
                                cursor_position,
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
                // Finnish connetion
                if let Some(start_position) = self.drag_start {
                    if let Some((final_position, _node_type)) = self
                        .state
                        .check_proximity_to_nodes(position, &start_position)
                    {
                        if let Some((target_node_index, _)) =
                            self.state.find_node_at_position(final_position.into())
                        {
                            return (
                                Status::Captured,
                                Some(Message::AddConnection(target_node_index)),
                            );
                        }
                    }
                }

                // Draw current dragging line
                if let Some(current_path) = &self.current_dragging_line {
                    if let Some(last_position) = current_path.last_point() {
                        if self.is_dragging {
                            return (
                                Status::Captured,
                                Some(Message::UpdateDraggingLine(position, *last_position)),
                            );
                        }
                    }
                    return (Status::Ignored, None);
                }

                // Means we are dragging from the node so we can set dragging to true
                if self.state.find_node_at_position(position) == None {
                    return (Status::Captured, Some(Message::UpdateIsDragging));
                }

                // Drag gates around canvas
                if let Some(index) = self.dragging_gate_index {
                    if let Some(offset) = self.drag_start {
                        return (
                            Status::Captured,
                            Some(Message::UpdateDraggingGatePosition(position, index, offset)),
                        );
                    }
                }
            }
            canvas::Event::Mouse(Event::ButtonReleased(Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((node_index, node_type)) = &self.dragging_node {
                        if *node_type == NodeType::Input
                            && self.state.find_node_at_position(cursor_position) != None
                        {
                            return (
                                Status::Captured,
                                Some(Message::UpdateNodeState(*node_index, node_type.clone())),
                            );
                        }
                    }
                    return (Status::Captured, Some(Message::DisabledDragging));
                }
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

        // Connections on the go
        canvas_connection_on_the_fly(&mut frame, &self.current_dragging_line);

        canvas_connections(&mut frame, &self.state.connections);

        vec![frame.into_geometry()]
    }
}
