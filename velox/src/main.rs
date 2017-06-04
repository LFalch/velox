extern crate piston_window;
extern crate find_folder;
extern crate velox_core;

use piston_window::*;

use game::SpaceShooter;
use std::env::args;

pub use velox_core::obj::Vector2;

mod game;

fn main() {
    let window: PistonWindow =
    WindowSettings::new(format!("Space Shooter {}", env!("CARGO_PKG_VERSION")), [1200, 900])
            .exit_on_esc(true)
            .vsync(true)
            .build().unwrap();
    let server = args().nth(1).unwrap_or_else(|| "127.0.0.1:7351".to_owned());
    let this = SpaceShooter::new(window, &server);
    this.start_network_thread();
    this.run();
}
