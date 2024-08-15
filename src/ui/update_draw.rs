use iced::{event::Status, Point, Rectangle};

use crate::helpers::helpers::DEFAULT_MARGIN;

use super::logic_gate_app::Message;

pub fn create_node(cursor_position: Point, bounds: Rectangle) -> (Status, Option<Message>) {
    if cursor_position.x <= DEFAULT_MARGIN {
        return (
            Status::Captured,
            Some(Message::AddInputNode(cursor_position)),
        );
    }

    if cursor_position.x >= bounds.width - DEFAULT_MARGIN {
        return (
            Status::Captured,
            Some(Message::AddOutputNode(cursor_position, bounds)),
        );
    }

    (Status::Ignored, None)
}
