#[macro_use]
extern crate korome;
extern crate velox_core;

macro_rules! when{
    ($info:expr; $($state:expr, $key:ident => $b:block),+) => {
        for ke in $info.get_key_events(){
            match *ke{
                $(($state, ::korome::VirtualKeyCode::$key) => $b,)+
                _ => ()
            }
        }
    };
}
macro_rules! when_mouse {
    ($info:expr; $($state:expr, $key:ident => $b:block),+) => {
        for ke in $info.get_mouse_events(){
            match *ke{
                $(($state, ::korome::MouseButton::$key) => $b,)+
                _ => ()
            }
        }
    };
}

use korome::*;
use game::SpaceShooter;
use std::env::args;

mod game;

fn main() {
    let graphics = Graphics::new("Space Shooter WIP", 1200, 900).unwrap();
    let server = args().nth(1).unwrap_or_else(|| "127.0.0.1:7351".to_owned());
    let this = SpaceShooter::new(&graphics, &server);
    this.start_network_thread();
    run_until_closed(graphics, this);
}