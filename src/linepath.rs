use serde::{Deserialize, Serialize};

use crate::serialize_point::SerializablePoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinePath {
    pub points: Vec<SerializablePoint>, // Store all points along the line
}

impl LinePath {
    pub fn new(start_point: SerializablePoint) -> Self {
        Self {
            points: vec![start_point],
        }
    }

    pub fn add_point(&mut self, point: SerializablePoint) {
        self.points.push(point);
    }

    pub fn last_point(&self) -> Option<&SerializablePoint> {
        self.points.last()
    }
}
