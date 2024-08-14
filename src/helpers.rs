use iced::{
    widget::canvas::{self, path::Arc, Frame, Path},
    Color,
};

pub const NODE_RADIUS: f32 = 15.0;
pub const SMALL_NODE_RADIUS: f32 = 5.0;
pub const LINE_WIDTH: f32 = 3.0;
pub const CORNER_RADIUS: f32 = 3.0;
pub const DEFAULT_MARGIN: f32 = 25.0;
pub const MIN_DISTANCE: f32 = 10.0;
pub const PROXIMITY_THRESHOLD: f32 = 20.0;

pub fn is_point_near(p1: iced::Point, p2: iced::Point, threshold: f32) -> bool {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt() <= threshold
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
        // Horizontal to vertical transition
        if next_point.y > corner_point.y {
            // Downward curve
            (
                iced::Point::new(corner_point.x + radius, corner_point.y + radius),
                1.5 * std::f32::consts::PI,
                2.0 * std::f32::consts::PI,
            )
        } else {
            // Upward curve
            (
                iced::Point::new(corner_point.x + radius, corner_point.y - radius),
                0.0,
                0.5 * std::f32::consts::PI,
            )
        }
    } else {
        // Vertical to horizontal transition
        if next_point.x > corner_point.x {
            // Rightward curve
            (
                iced::Point::new(corner_point.x + radius, corner_point.y + radius),
                std::f32::consts::PI,
                1.5 * std::f32::consts::PI,
            )
        } else {
            // Leftward curve
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
