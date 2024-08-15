use iced::{
    widget::canvas::{self, path::Arc, Frame, Path, Stroke},
    Color,
};

use crate::components::{gate::LogicGate, node::Node};

pub const NODE_RADIUS: f32 = 15.0;
pub const SMALL_NODE_RADIUS: f32 = 5.0;
pub const LINE_WIDTH: f32 = 3.0;
pub const DEFAULT_MARGIN: f32 = 25.0;
pub const MIN_DISTANCE: f32 = 5.0;
pub const DIRECTION_CHANGE_THRESHOLD: f32 = 600.0;

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

pub fn draw_smooth_corner_curve(
    frame: &mut Frame,
    start_point: iced::Point,
    end_point: iced::Point,
    next_point: iced::Point,
    line_width: f32,
    color: Color,
) {
    // Calculate control points for cubic Bézier curve
    let control_point1 = iced::Point::new((start_point.x + end_point.x) / 2.0, start_point.y);
    let control_point2 = iced::Point::new((end_point.x + next_point.x) / 2.0, next_point.y);

    let smooth_curve = Path::new(|path| {
        path.move_to(start_point);
        path.bezier_curve_to(control_point1, control_point2, end_point);
        path.line_to(next_point);
    });

    frame.stroke(
        &smooth_curve,
        Stroke::default().with_width(line_width).with_color(color),
    );
}
