use iced::{
    widget::canvas::{self, path::Arc, Frame, Path},
    Color,
};

use crate::components::{gate::LogicGate, node::Node};

pub const NODE_RADIUS: f32 = 15.0;
pub const SMALL_NODE_RADIUS: f32 = 5.0;
pub const LINE_WIDTH: f32 = 3.0;
pub const CORNER_RADIUS: f32 = 3.0;
pub const DEFAULT_MARGIN: f32 = 25.0;
pub const MIN_DISTANCE: f32 = 10.0;
pub const PROXIMITY_THRESHOLD: f32 = 10.0;

pub const MIN_GATE_WIDTH: f32 = 50.0;
pub const MIN_GATE_HEIGHT: f32 = 30.0;

pub fn calculate_gate_size(num_inputs: usize, num_outputs: usize) -> (f32, f32) {
    let node_size = 10.0;
    let height = MIN_GATE_HEIGHT.max((num_inputs.max(num_outputs) as f32) * node_size);

    let width = MIN_GATE_WIDTH;

    (width, height)
}

pub fn is_point_near_gate(p: iced::Point, gate: &LogicGate) -> bool {
    p.x >= gate.position.x
        && p.x <= gate.position.x + gate.width
        && p.y >= gate.position.y
        && p.y <= gate.position.y + gate.height
}

pub fn is_point_near_node(p: iced::Point, node: &Node) -> bool {
    let dx = p.x - node.position.x;
    let dy = p.y - node.position.y;
    let distance = (dx * dx + dy * dy).sqrt();
    distance <= node.radius
}

pub fn draw_corner_arc(
    frame: &mut Frame,
    start_point: iced::Point,
    end_point: iced::Point,
    next_point: iced::Point,
    line_width: f32,
    color: Color,
    radius: f32,
) {
    let is_horizontal = (start_point.x - end_point.x).abs() > (start_point.y - end_point.y).abs();

    let corner_point = end_point;

    let (arc_center, start_angle, end_angle) = if is_horizontal {
        if next_point.y > corner_point.y {
            (
                iced::Point::new(corner_point.x + radius, corner_point.y + radius),
                1.5 * std::f32::consts::PI,
                2.0 * std::f32::consts::PI,
            )
        } else {
            (
                iced::Point::new(corner_point.x + radius, corner_point.y - radius),
                0.0,
                0.5 * std::f32::consts::PI,
            )
        }
    } else {
        if next_point.x > corner_point.x {
            (
                iced::Point::new(corner_point.x + radius, corner_point.y + radius),
                std::f32::consts::PI,
                1.5 * std::f32::consts::PI,
            )
        } else {
            (
                iced::Point::new(corner_point.x - radius, corner_point.y + radius),
                0.5 * std::f32::consts::PI,
                std::f32::consts::PI,
            )
        }
    };

    let corner_arc = Path::new(|path| {
        path.arc(Arc {
            center: arc_center,
            radius,
            start_angle: iced::Radians(start_angle),
            end_angle: iced::Radians(end_angle),
        });
    });

    frame.stroke(
        &corner_arc,
        canvas::Stroke::default()
            .with_width(line_width)
            .with_color(color),
    );
}
