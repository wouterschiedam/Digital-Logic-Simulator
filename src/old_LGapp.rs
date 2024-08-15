use iced::{
    event::Status,
    mouse::{self, Cursor},
    widget::canvas::{self, Frame, Path, Program},
    Color, Rectangle, Renderer, Theme,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell, RefMut},
    fs::File,
    io::{Read, Write},
};

use crate::{
    gates::{GateType, LogicGate},
    helpers::{
        calculate_gate_size, draw_corner_arc, is_point_near, CORNER_RADIUS, DEFAULT_MARGIN,
        LINE_WIDTH, MIN_DISTANCE, NODE_RADIUS, SMALL_NODE_RADIUS,
    },
    linepath::LinePath,
    nodes::{Node, NodeType, Nodes},
    serialize_point::SerializablePoint,
    ui::Message,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub is_active: bool, // Track whether the connection is active (carrying current)
    pub path: Vec<SerializablePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicGateApp {
    pub nodes: RefCell<Vec<Nodes>>,
    pub gates: RefCell<Option<Vec<LogicGate>>>,
    pub connections: RefCell<Vec<Connection>>, // (source node index, target gate index)
    pub dragging_node: RefCell<Option<(usize, NodeType)>>, // Track the node being dragged (node index, node type)
    pub drag_start: RefCell<Option<SerializablePoint>>,    // Starting point of the drag
    pub current_drag_position: RefCell<Option<SerializablePoint>>, // Current mouse position during the drag
    pub current_dragging_line: RefCell<Option<LinePath>>,
    pub is_dragging: Cell<bool>,
    pub dragging_gate_index: RefCell<Option<usize>>,
}

impl LogicGateApp {
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;
        let app_state: LogicGateApp = serde_json::from_str(&json)?;
        Ok(app_state)
    }

    pub fn add_gate(
        &mut self,
        gate_type: GateType,
        position: SerializablePoint,
        input: usize,
        output: usize,
    ) {
        let gate = LogicGate::new(gate_type, position, input, output);
        let mut gates_ref = self.gates.borrow_mut();

        if let Some(gates_vec) = gates_ref.as_mut() {
            gates_vec.push(gate);
        } else {
            *gates_ref = Some(vec![gate]);
        }
    }

    pub fn update_connections(
        connections: &mut [Connection],
        nodes: &mut [Nodes],
        gates: &mut Option<Vec<LogicGate>>,
    ) {
        // First, gather the nodes' output states and the corresponding connections to update
        let mut updates = Vec::new();

        for (node_index, node) in nodes.iter().enumerate() {
            if let Some(output) = node.input_nodes.get(0) {
                if output.state {
                    // If output is active, prepare to activate the corresponding connections
                    for connection in connections.iter_mut() {
                        if connection.from == node_index {
                            updates.push((connection.to, true)); // Mark the target node input to activate
                            connection.is_active = true; // Mark the connection as active
                        }
                    }
                } else {
                    // If output is inactive, prepare to deactivate the corresponding connections
                    for connection in connections.iter_mut() {
                        if connection.from == node_index {
                            updates.push((connection.to, false)); // Mark the target node input to deactivate
                            connection.is_active = false; // Mark the connection as inactive
                        }
                    }
                }
            }
        }

        // Now, perform the updates with a mutable borrow
        for (node_index, state) in updates {
            // Update the input state of the node receiving the input
            if let Some(input) = nodes[node_index].input_nodes.get_mut(0) {
                input.state = state;
            }
            // Update the output state of the node receiving the input
            if let Some(output) = nodes[node_index].output_nodes.get_mut(0) {
                output.state = state;
            }

            // Check if the node is connected to a gate and perform gate logic
            if let Some(gates_vec) = gates {
                for gate in gates_vec.iter_mut() {
                    // Find the input node of the gate and compare its position with the node being updated
                    if let Some(gate_input_node) = gate.nodes.input_nodes.get(0) {
                        // Compare based on position or some unique identifier if available
                        if gate_input_node.position == nodes[node_index].input_nodes[0].position {
                            // If the gate is a NOT gate, negate the output
                            if gate.gate_type == GateType::Not {
                                if let Some(output_node) = gate.nodes.output_nodes.get_mut(0) {
                                    output_node.state = !state; // Negate the input state for the NOT gate
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_node_at_position(
        &self,
        position: iced::Point,
        nodes: &mut RefMut<Vec<Nodes>>,
    ) -> Option<(usize, NodeType)> {
        // Iterate over each node and its input_nodes/output_nodes
        for (node_index, node) in nodes.iter().enumerate() {
            // Check the input nodes
            for input in &node.input_nodes {
                let node_position: iced::Point = input.position.clone().into();
                if is_point_near(position, node_position, NODE_RADIUS) {
                    return Some((node_index, NodeType::Input));
                }
            }

            // Check the output nodes
            for output in &node.output_nodes {
                let node_position: iced::Point = output.position.clone().into();
                if is_point_near(position, node_position, NODE_RADIUS) {
                    return Some((node_index, NodeType::Output));
                }
            }
        }

        // Return None if no node is found near the given position
        None
    }

    pub fn check_proximity_to_nodes(
        &self,
        cursor_position: iced::Point,
        nodes: &mut RefMut<Vec<Nodes>>,
        gates: &mut Option<Vec<LogicGate>>,
        start_position: &SerializablePoint,
    ) -> Option<SerializablePoint> {
        // First check overal free nodes
        for node in nodes.iter() {
            for node in &node.input_nodes {
                let node_position: iced::Point = node.position.clone().into();
                if node.position != *start_position
                    && is_point_near(cursor_position, node_position, 1.0)
                {
                    return Some(node.position.clone());
                }
            }

            for node in &node.output_nodes {
                let node_position: iced::Point = node.position.clone().into();
                if node.position != *start_position
                    && is_point_near(cursor_position, node_position, 5.0)
                {
                    return Some(node.position.clone());
                }
            }
        }

        // Now check nodes inside gates
        if let Some(gates_vec) = gates {
            for gate in gates_vec.iter() {
                for input_node in &gate.nodes.input_nodes {
                    let node_position: iced::Point = input_node.position.clone().into();
                    if input_node.position != *start_position
                        && is_point_near(cursor_position, node_position, 1.0)
                    {
                        return Some(input_node.position.clone());
                    }
                }

                for output_node in &gate.nodes.output_nodes {
                    let node_position: iced::Point = output_node.position.clone().into();
                    if output_node.position != *start_position
                        && is_point_near(cursor_position, node_position, 5.0)
                    {
                        return Some(output_node.position.clone());
                    }
                }
            }
        }

        None
    }
}

// Implementing the Program trait for LogicGateApp
impl Program<Message, Theme, Renderer> for LogicGateApp {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<Message>) {
        let mut nodes = self.nodes.borrow_mut();
        let mut gates = self.gates.borrow_mut();

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some(gates) = gates.as_ref() {
                        for (index, gate) in gates.iter().enumerate() {
                            let gate_position: iced::Point = gate.position.clone().into();
                            let gate_rect = Rectangle {
                                x: gate_position.x,
                                y: gate_position.y,
                                width: 50.0,
                                height: 30.0,
                            };

                            if gate_rect.contains(cursor_position) {
                                let offset = iced::Point {
                                    x: cursor_position.x - gate_position.x,
                                    y: cursor_position.y - gate_position.y,
                                };

                                // Gate is clicked, start dragging
                                self.is_dragging.set(true);
                                *self.dragging_gate_index.borrow_mut() = Some(index);
                                *self.drag_start.borrow_mut() = Some(offset.into());
                                return (iced::event::Status::Captured, None);
                            }
                        }
                    }

                    for (node_index, node) in nodes.iter_mut().enumerate() {
                        for input in &mut node.input_nodes {
                            if is_point_near(
                                cursor_position,
                                input.position.clone().into(),
                                NODE_RADIUS,
                            ) {
                                *self.current_dragging_line.borrow_mut() =
                                    Some(LinePath::new(SerializablePoint {
                                        x: cursor_position.x,
                                        y: cursor_position.y,
                                    }));
                                *self.dragging_node.borrow_mut() =
                                    Some((node_index, NodeType::Input));
                                *self.drag_start.borrow_mut() = Some(input.position.clone());
                                self.is_dragging.set(false);
                                return (Status::Captured, None);
                            }
                        }
                        for output in &mut node.output_nodes {
                            if is_point_near(
                                cursor_position,
                                output.position.clone().into(),
                                NODE_RADIUS,
                            ) {
                                *self.current_dragging_line.borrow_mut() =
                                    Some(LinePath::new(SerializablePoint {
                                        x: cursor_position.x,
                                        y: cursor_position.y,
                                    }));
                                *self.dragging_node.borrow_mut() =
                                    Some((node_index, NodeType::Output));
                                *self.drag_start.borrow_mut() = Some(output.position.clone());
                                self.is_dragging.set(false);
                                return (Status::Captured, None);
                            }
                        }
                    }

                    // If clicking near the borders, add a node (existing functionality)
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
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let mut finalize_connection = false;
                let mut final_position = None;
                let mut start_node_index = None;
                if let Some(drag_start_point) = self.drag_start.borrow().as_ref() {
                    start_node_index =
                        self.find_node_at_position(drag_start_point.clone().into(), &mut nodes);
                }

                // Drag gates around canvas
                if let Some(index) = *self.dragging_gate_index.borrow() {
                    if let Some(gates) = gates.borrow_mut().as_mut() {
                        if let Some(gate) = gates.get_mut(index) {
                            if let Some(offset) = self.drag_start.borrow().as_ref() {
                                // Update gate position based on cursor movement
                                gate.position.x = position.x - offset.x;
                                gate.position.y = position.y - offset.y;

                                // Calculate the number of input and output nodes immutably first
                                let num_input_nodes = gate.nodes.input_nodes.len();
                                let num_output_nodes = gate.nodes.output_nodes.len();

                                // Update the positions of the nodes relative to the new gate position
                                let gate_width = gate.width;
                                let gate_height = gate.height;
                                let gate_position = gate.position.clone();

                                // Update input nodes
                                for (i, input_node) in gate.nodes.input_nodes.iter_mut().enumerate()
                                {
                                    let y_position = gate_position.y
                                        + (i as f32 + 1.0) * gate_height
                                            / (num_input_nodes as f32 + 1.0);
                                    input_node.position.x = gate_position.x;
                                    input_node.position.y = y_position;
                                }

                                // Update output nodes
                                for (i, output_node) in
                                    gate.nodes.output_nodes.iter_mut().enumerate()
                                {
                                    let y_position = gate_position.y
                                        + (i as f32 + 1.0) * gate_height
                                            / (num_output_nodes as f32 + 1.0);
                                    output_node.position.x = gate_position.x + gate_width;
                                    output_node.position.y = y_position;
                                }

                                return (iced::event::Status::Captured, None);
                            }
                        }
                    }
                }

                // Start dragging a line
                if let Some(current_path) = self.current_dragging_line.borrow_mut().as_mut() {
                    if let Some(last_position) = current_path.last_point() {
                        // Perform proximity check, skipping the start node
                        final_position = self.check_proximity_to_nodes(
                            position,
                            &mut nodes,
                            &mut gates,
                            last_position,
                        );

                        if final_position.is_some() {
                            finalize_connection = true;
                        } else {
                            let distance_x = (last_position.x - position.x).abs();
                            let distance_y = (last_position.y - position.y).abs();

                            if distance_x > MIN_DISTANCE || distance_y > MIN_DISTANCE {
                                self.is_dragging.set(true);

                                let new_point = if distance_x > distance_y {
                                    SerializablePoint {
                                        x: position.x,
                                        y: last_position.y,
                                    }
                                } else {
                                    SerializablePoint {
                                        x: last_position.x,
                                        y: position.y,
                                    }
                                };

                                current_path.add_point(new_point);
                            }
                        }
                    }
                }

                if finalize_connection {
                    if let Some(final_position) = final_position {
                        if let Some(current_path) = self.current_dragging_line.borrow_mut().as_mut()
                        {
                            current_path.add_point(final_position.clone());

                            // Log current path points
                            println!("Current Path Points: {:?}", current_path.points);

                            // First, check if a free node is the target
                            if let Some((target_node_index, _)) =
                                self.find_node_at_position(final_position.into(), &mut nodes)
                            {
                                let path = current_path.points.clone();
                                let start_index = start_node_index.unwrap().0; // Store the index separately to avoid moving it

                                self.connections.borrow_mut().push(Connection {
                                    from: start_index,
                                    to: target_node_index,
                                    is_active: false,
                                    path,
                                });
                            } else if let Some(gates_vec) = gates.as_ref() {
                                let mut connection_made = false;
                                let start_index = start_node_index.unwrap().0; // Store the index separately

                                for (gate_index, gate) in gates_vec.iter().enumerate() {
                                    // Check input nodes inside the gate
                                    for input_node in &gate.nodes.input_nodes {
                                        if is_point_near(
                                            final_position.into(),
                                            input_node.position.clone().into(),
                                            NODE_RADIUS,
                                        ) {
                                            let path = current_path.points.clone();

                                            // Log the indices
                                            println!(
                                                "Connection from: {:?} to (gate node): {:?}",
                                                start_index, gate_index
                                            );

                                            self.connections.borrow_mut().push(Connection {
                                                from: start_index,
                                                to: gate_index,
                                                is_active: false,
                                                path,
                                            });
                                            connection_made = true;
                                            break;
                                        }
                                    }
                                    if connection_made {
                                        break;
                                    }

                                    // Check output nodes inside the gate
                                    for output_node in &gate.nodes.output_nodes {
                                        if is_point_near(
                                            final_position.into(),
                                            output_node.position.clone().into(),
                                            NODE_RADIUS,
                                        ) {
                                            let path = current_path.points.clone();

                                            // Log the indices
                                            println!(
                                                "Connection from: {:?} to (gate node): {:?}",
                                                start_index, gate_index
                                            );

                                            self.connections.borrow_mut().push(Connection {
                                                from: start_index,
                                                to: gate_index,
                                                is_active: false,
                                                path,
                                            });
                                            connection_made = true;
                                            break;
                                        }
                                    }
                                    if connection_made {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    *self.current_dragging_line.borrow_mut() = None;
                    self.is_dragging.set(false);
                    return (iced::event::Status::Captured, None);
                }

                // // Finish connection line to node
                // if finalize_connection {
                //     if let Some(final_position) = final_position {
                //         if let Some(current_path) = self.current_dragging_line.borrow_mut().as_mut()
                //         {
                //             current_path.add_point(final_position.clone());
                //
                //             // Determine the target node index
                //             if let Some((target_node_index, _)) =
                //                 self.find_node_at_position(final_position.into(), &mut nodes)
                //             {
                //                 // Store the connection with the custom path
                //                 let path = current_path.points.clone();
                //                 self.connections.borrow_mut().push(Connection {
                //                     from: start_node_index.unwrap().0,
                //                     to: target_node_index,
                //                     is_active: false,
                //                     path,
                //                 });
                //             }
                //         }
                //     }
                //
                //     *self.current_dragging_line.borrow_mut() = None;
                //     self.is_dragging.set(false);
                //     return (iced::event::Status::Captured, None);
                // }

                return (iced::event::Status::Captured, None);
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.is_dragging.get() {
                    *self.current_dragging_line.borrow_mut() = None;
                    *self.dragging_gate_index.borrow_mut() = None;

                    // extracted the start_node_index using map() to immediately get the value and drop the Ref borrow as soon as possible.
                    let start_node_index_opt = {
                        let start_node = self.dragging_node.borrow();
                        start_node
                            .as_ref()
                            .map(|(start_node_index, _)| *start_node_index)
                    };

                    if let Some(start_node_index) = start_node_index_opt {
                        if let Some(cursor_position) = cursor.position_in(bounds) {
                            let _cursor_point = SerializablePoint {
                                x: cursor_position.x,
                                y: cursor_position.y,
                            };

                            let mut connection_to_add = None;

                            for (node_index, node) in nodes.iter_mut().enumerate() {
                                for input in &mut node.input_nodes {
                                    if is_point_near(
                                        cursor_position,
                                        input.position.clone().into(),
                                        NODE_RADIUS,
                                    ) {
                                        connection_to_add = Some(Connection {
                                            from: start_node_index,
                                            to: node_index,
                                            is_active: false,
                                            path: Vec::new(),
                                        });
                                        break;
                                    }
                                }
                                for output in &mut node.output_nodes {
                                    if is_point_near(
                                        cursor_position,
                                        output.position.clone().into(),
                                        NODE_RADIUS,
                                    ) {
                                        connection_to_add = Some(Connection {
                                            from: start_node_index,
                                            to: node_index,
                                            is_active: false,
                                            path: Vec::new(),
                                        });
                                        break;
                                    }
                                }
                                if connection_to_add.is_some() {
                                    break;
                                }
                            }

                            if let Some(gates_vec) = gates.as_mut() {
                                for (gate_index, gate) in gates_vec.iter_mut().enumerate() {
                                    for node in &gate.nodes.input_nodes {
                                        if is_point_near(
                                            cursor_position,
                                            node.position.clone().into(),
                                            NODE_RADIUS,
                                        ) {
                                            connection_to_add = Some(Connection {
                                                from: start_node_index,
                                                to: gate_index,
                                                is_active: false,
                                                path: Vec::new(),
                                            });
                                            break;
                                        }
                                    }
                                }
                            } else {
                                println!("No gates found.");
                            }

                            if let Some(connection) = connection_to_add {
                                {
                                    // Push the connection to the connections vector
                                    let mut connections = self.connections.borrow_mut();
                                    connections.push(connection);
                                }

                                // Now perform the update using a new mutable borrow
                                LogicGateApp::update_connections(
                                    &mut self.connections.borrow_mut(),
                                    &mut nodes,
                                    &mut gates,
                                );
                            }
                        }
                    }
                } else {
                    // Handle a click (toggle the node state)
                    if let Some((node_index, node_type)) = self.dragging_node.borrow().as_ref() {
                        let node = &mut nodes[*node_index];
                        match node_type {
                            NodeType::Input => {
                                if let Some(node) = node.input_nodes.iter_mut().find(|n| {
                                    is_point_near(
                                        cursor.position_in(bounds).unwrap(),
                                        n.position.clone().into(),
                                        NODE_RADIUS,
                                    )
                                }) {
                                    node.state = !node.state;
                                    LogicGateApp::update_connections(
                                        &mut self.connections.borrow_mut(),
                                        &mut nodes,
                                        &mut gates,
                                    );
                                }
                            }
                            NodeType::Output => {
                                if let Some(node) = node.output_nodes.iter_mut().find(|n| {
                                    is_point_near(
                                        cursor.position_in(bounds).unwrap(),
                                        n.position.clone().into(),
                                        NODE_RADIUS,
                                    )
                                }) {
                                    node.state = !node.state;
                                    LogicGateApp::update_connections(
                                        &mut self.connections.borrow_mut(),
                                        &mut nodes,
                                        &mut gates,
                                    );
                                }
                            }
                        }
                    }
                }
                *self.current_drag_position.borrow_mut() = None;
                *self.drag_start.borrow_mut() = None;
                *self.dragging_node.borrow_mut() = None;
                self.is_dragging.set(false);
                return (Status::Captured, None);
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
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(&renderer, bounds.size());

        // Define margin size
        let margin = DEFAULT_MARGIN;

        // Calculate the size and position of the rectangle
        let rect_position = iced::Point::new(margin, margin);
        let rect_size = iced::Size::new(bounds.width - 2.0 * margin, bounds.height - 2.0 * margin);

        // Create the rectangle path
        let rect_path = canvas::Path::rectangle(rect_position, rect_size);

        frame.stroke(
            &rect_path,
            canvas::Stroke::default()
                .with_width(LINE_WIDTH)
                .with_color(Color::from_rgb(0.7, 0.7, 0.7)),
        );

        let nodes = self.nodes.borrow();

        for node in nodes.iter() {
            for input in &node.input_nodes {
                let node_shape = canvas::Path::circle(input.position.clone().into(), NODE_RADIUS);
                if input.state {
                    frame.fill(&node_shape, Color::from_rgb(1.0, 0.0, 0.0));
                } else {
                    frame.fill(&node_shape, Color::from_rgb(0.3, 0.3, 0.3));
                }
            }
            for output in &node.output_nodes {
                let node_shape = canvas::Path::circle(output.position.clone().into(), NODE_RADIUS);
                if output.state {
                    frame.fill(&node_shape, Color::from_rgb(1.0, 0.0, 0.0));
                } else {
                    frame.fill(&node_shape, Color::from_rgb(0.3, 0.3, 0.3));
                }
            }
        }

        let gates = self.gates.borrow();
        if let Some(gates_vec) = gates.as_ref() {
            for gate in gates_vec.iter() {
                let position: iced::Point = gate.position.clone().into();
                let gate_shape =
                    Path::rectangle(position, iced::Size::new(gate.width, gate.height)); // Use gate-specific size
                frame.fill(&gate_shape, Color::from_rgb(0.7, 0.7, 0.7)); // Fill color for gate

                // Draw input nodes
                for node in &gate.nodes.input_nodes {
                    let input_shape = Path::circle(node.position.clone().into(), SMALL_NODE_RADIUS);
                    frame.fill(&input_shape, Color::BLACK);
                }

                // Draw output nodes
                for node in &gate.nodes.output_nodes {
                    let output_shape =
                        Path::circle(node.position.clone().into(), SMALL_NODE_RADIUS);
                    frame.fill(&output_shape, Color::BLACK);
                }

                // Draw gate label
                let gate_name = match gate.gate_type {
                    GateType::And => "AND",
                    GateType::Or => "OR",
                    GateType::Not => "NOT",
                };

                let text_position = iced::Point::new(position.x + 8.0, position.y + 5.0); // Adjust position as needed
                frame.fill_text(canvas::Text {
                    content: gate_name.to_string(),
                    position: text_position,
                    color: Color::BLACK,
                    size: iced::Pixels(16.0), // Font size
                    ..canvas::Text::default()
                });

                // Draw output indicator
                let output_pos =
                    iced::Point::new(position.x + gate.width, position.y + gate.height / 2.0); // Adjust for gate size
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

        if let Some(current_path) = self.current_dragging_line.borrow().as_ref() {
            if current_path.points.len() > 1 {
                for i in 0..current_path.points.len() - 1 {
                    let start_point: iced::Point = current_path.points[i].clone().into();
                    let end_point: iced::Point = current_path.points[i + 1].clone().into();

                    // Draw each segment of the path
                    frame.stroke(
                        &Path::line(start_point, end_point),
                        canvas::Stroke::default()
                            .with_width(LINE_WIDTH)
                            .with_color(Color::BLACK),
                    );

                    if i < current_path.points.len() - 2 {
                        let next_point: iced::Point = current_path.points[i + 2].clone().into();

                        let current_direction =
                            (end_point.x - start_point.x, end_point.y - start_point.y);
                        let next_direction =
                            (next_point.x - end_point.x, next_point.y - end_point.y);

                        let is_direction_change = (current_direction.0 != 0.0
                            && next_direction.1 != 0.0)
                            || (current_direction.1 != 0.0 && next_direction.0 != 0.0);

                        if is_direction_change {
                            // Draw the corner arc if the direction changes
                            draw_corner_arc(
                                &mut frame,
                                start_point,
                                end_point,
                                next_point,
                                LINE_WIDTH,
                                Color::BLACK,
                                CORNER_RADIUS,
                            );
                        }
                    }
                }
            }
        }

        // Draw existing connections using the stored path
        for connection in self.connections.borrow().iter() {
            if connection.path.len() > 1 {
                for i in 0..connection.path.len() - 1 {
                    let start_point: iced::Point = connection.path[i].clone().into();
                    let end_point: iced::Point = connection.path[i + 1].clone().into();

                    frame.stroke(
                        &Path::line(start_point, end_point),
                        canvas::Stroke::default().with_width(LINE_WIDTH).with_color(
                            if connection.is_active {
                                Color::from_rgb(1.0, 0.0, 0.0) // Active connections are red
                            } else {
                                Color::from_rgb(0.0, 0.0, 0.0) // Inactive connections are black
                            },
                        ),
                    );
                }
            }
        }

        vec![frame.into_geometry()]
    }
}
