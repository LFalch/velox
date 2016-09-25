#[macro_use]
extern crate korome;
extern crate simple_vector2d;

use korome::*;

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

mod game;
use game::SpaceShooterBuilder;

macro_rules! textures {
    ($graphics:ident; $($tex:ident),*) => ($(
        let $tex = match Texture::from_file(&$graphics, concat!("tex/", stringify!($tex), ".png")){
            Ok(t) => t,
            Err(_) => return println!(concat!("Failed to load texture ", stringify!($tex)))
        }
    )*);
}

fn main() {
    let graphics = Graphics::new("Space Shooter WIP", 1200, 900).unwrap();
    textures!(graphics; planet, ship, sun, arrow, laser);
    let this = SpaceShooterBuilder{
        planet: &planet,
        ship: &ship,
        sun: &sun,
        arrow: &arrow,
        laser: &laser
    }.finish();

    run_until_closed(graphics, this);
}
