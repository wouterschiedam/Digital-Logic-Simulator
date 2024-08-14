use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializablePoint {
    pub x: f32,
    pub y: f32,
}

impl From<iced::Point> for SerializablePoint {
    fn from(point: iced::Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<SerializablePoint> for iced::Point {
    fn from(s_point: SerializablePoint) -> Self {
        Self {
            x: s_point.x,
            y: s_point.y,
        }
    }
}
