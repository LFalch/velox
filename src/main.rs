#[macro_use]
extern crate korome;

use korome::*;

mod game;
use game::SpaceShooter;

mod obj;

pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 900;

macro_rules! textures {
    ($graphics:ident; $($tex:ident),*) => ($(
        let $tex = Texture::from_png_bytes(&$graphics, include_bytes!(concat!(stringify!($tex), ".png"))).unwrap();
    )*);
}

fn main() {
    let graphics = Graphics::new("SPACE-SHOOTER", WIDTH, HEIGHT).unwrap();

    textures!(graphics; planet, ship, sun, arrow);

    let this = SpaceShooter::new(&planet, &ship, &sun, &arrow);

    run_until_closed(graphics, this);
}
