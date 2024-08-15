use iced::{
    widget::canvas::{Frame, Path, Stroke, Text},
    Color, Point, Rectangle,
};

use crate::{
    components::{
        connection::Connection,
        gate::{GateType, LogicGate},
        line_path::LinePath,
        node::Nodes,
    },
    helpers::helpers::{
        draw_smooth_corner_curve, DEFAULT_MARGIN, DIRECTION_CHANGE_THRESHOLD, LINE_WIDTH,
        NODE_RADIUS, SMALL_NODE_RADIUS,
    },
};

pub fn canvas_frame(frame: &mut Frame, bounds: Rectangle) {
    // Define margin size
    let margin = DEFAULT_MARGIN;

    // Calculate the size and position of the rectangle
    let rect_position = iced::Point::new(margin, margin);

    let rect_size = iced::Size::new(bounds.width - 2.0 * margin, bounds.height - 2.0 * margin);

    // Create the rectangle path
    let rect_path = Path::rectangle(rect_position, rect_size);

    frame.stroke(
        &rect_path,
        Stroke::default()
            .with_width(LINE_WIDTH)
            .with_color(Color::from_rgb(0.7, 0.7, 0.7)),
    );
}

pub fn canvas_free_nodes(frame: &mut Frame, nodes: &Vec<Nodes>) {
    for node in nodes.iter() {
        for input in &node.input_nodes {
            let node_shape = Path::circle(input.position.clone().into(), NODE_RADIUS);
            if input.state {
                frame.fill(&node_shape, Color::from_rgb(1.0, 0.0, 0.0));
            } else {
                frame.fill(&node_shape, Color::from_rgb(0.3, 0.3, 0.3));
            }
        }
        for output in &node.output_nodes {
            let node_shape = Path::circle(output.position.clone().into(), NODE_RADIUS);
            if output.state {
                frame.fill(&node_shape, Color::from_rgb(1.0, 0.0, 0.0));
            } else {
                frame.fill(&node_shape, Color::from_rgb(0.3, 0.3, 0.3));
            }
        }
    }
}

pub fn canvas_gates(frame: &mut Frame, gates: &Vec<LogicGate>) {
    for gate in gates {
        let position: iced::Point = gate.position.clone().into();
        let gate_shape = Path::rectangle(position, iced::Size::new(gate.width, gate.height)); // Use gate-specific size
        frame.fill(&gate_shape, Color::from_rgb(0.7, 0.7, 0.7)); // Fill color for gate

        // Draw input nodes
        for node in &gate.nodes.input_nodes {
            let input_shape = Path::circle(node.position.clone().into(), SMALL_NODE_RADIUS);
            frame.fill(&input_shape, Color::BLACK);
        }

        // Draw output nodes
        for node in &gate.nodes.output_nodes {
            let output_shape = Path::circle(node.position.clone().into(), SMALL_NODE_RADIUS);
            frame.fill(&output_shape, Color::BLACK);
        }

        // Draw gate label
        let gate_name = match gate.gate_type {
            GateType::And => "AND",
            GateType::Not => "NOT",
        };

        let text_position = iced::Point::new(position.x + 8.0, position.y + 5.0); // Adjust position as needed
        frame.fill_text(Text {
            content: gate_name.to_string(),
            position: text_position,
            color: Color::BLACK,
            size: iced::Pixels(16.0), // Font size
            ..Text::default()
        });

        // Draw output indicator
        let output_pos = iced::Point::new(position.x + gate.width, position.y + gate.height / 2.0); // Adjust for gate size
        let output_shape = Path::circle(output_pos, 5.0);
        frame.fill(
            &output_shape,
            if gate.get_output() {
                Color::WHITE
            } else {
                Color::BLACK
            },
        );
    }
}

pub fn canvas_connections(frame: &mut Frame, connections: &Vec<Connection>) {
    for connection in connections.iter() {
        if connection.path.len() > 1 {
            for i in 0..connection.path.len() - 1 {
                let start_point: iced::Point = connection.path[i].clone().into();
                let end_point: iced::Point = connection.path[i + 1].clone().into();

                frame.stroke(
                    &Path::line(start_point, end_point),
                    Stroke::default()
                        .with_width(LINE_WIDTH)
                        .with_color(if connection.is_active {
                            Color::from_rgb(1.0, 0.0, 0.0) // Active connections are red
                        } else {
                            Color::from_rgb(0.0, 0.0, 0.0) // Inactive connections are black
                        }),
                );
            }
        }
    }
}

pub fn canvas_connection_on_the_fly(frame: &mut Frame, line: &Option<LinePath>) {
    if let Some(current_path) = line {
        if current_path.points.len() > 1 {
            for i in 0..current_path.points.len() - 1 {
                let start_point = current_path.points[i].clone().into();
                let end_point = current_path.points[i + 1].clone().into();

                // Draw each segment of the path
                frame.stroke(
                    &Path::line(start_point, end_point),
                    Stroke::default()
                        .with_width(LINE_WIDTH)
                        .with_color(Color::BLACK),
                );

                if i < current_path.points.len() - 2 {
                    let next_point: Point = current_path.points[i + 2].clone().into();

                    let current_direction =
                        (end_point.x - start_point.x, end_point.y - start_point.y);
                    let next_direction = (next_point.x - end_point.x, next_point.y - end_point.y);

                    let direction_change_magnitude = (current_direction.0 * next_direction.1
                        - current_direction.1 * next_direction.0)
                        .abs();

                    let is_direction_change =
                        direction_change_magnitude > DIRECTION_CHANGE_THRESHOLD;

                    if is_direction_change {
                        // Draw the smooth corner curve if the direction changes significantly
                        draw_smooth_corner_curve(
                            frame,
                            start_point,
                            end_point,
                            next_point,
                            LINE_WIDTH,
                            Color::BLACK,
                        );
                    }
                }
            }
        }
    }
}
