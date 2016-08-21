#[macro_use]
extern crate korome;
extern crate simple_vector2d;

use std::env::args;

use korome::*;

pub use simple_vector2d::Vector2;

mod game;
use game::SpaceShooter;

mod obj;

macro_rules! textures {
    ($graphics:ident; $($tex:ident),*) => ($(
        let $tex = Texture::from_png_bytes(&$graphics, include_bytes!(concat!("tex/", stringify!($tex), ".png"))).unwrap();
    )*);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum OutOfBoundsBehaviour{
    Wrap, Bounce, Stop
}

fn main() {
    let graphics = Graphics::new("SPACE-SHOOTER", 1200, 900).unwrap();

    textures!(graphics; planet, ship, sun, arrow, laser);

    let this = SpaceShooter::new(args().any(|a| a == "--deltas"), &planet, &ship, &sun, &arrow, &laser);

    run_until_closed(graphics, this);
}
