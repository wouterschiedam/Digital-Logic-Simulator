// use ui::run;
//
// mod LGapp;
// mod gates;
// mod helpers;
// mod linepath;
// mod nodes;
// mod serialize_point;
// mod ui;
//
// fn main() {
//     let _ = run();
// }
//
//

mod components;
mod config;
mod helpers;
mod serialize_point;
mod state;
mod ui;

use ui::logic_gate_app::run;

fn main() {
    let _ = run();
}
