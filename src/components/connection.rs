use crate::serialize_point::SerializablePoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub is_active: bool,
    pub path: Vec<SerializablePoint>,
}

impl Connection {
    pub fn new(from: usize, to: usize, path: Vec<SerializablePoint>) -> Self {
        Self {
            from,
            to,
            is_active: false,
            path,
        }
    }
}
