use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub struct SerializablePoint {
    pub x: f32,
    pub y: f32,
}

impl SerializablePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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
