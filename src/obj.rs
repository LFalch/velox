pub type Vect = Vector2<f32>;
pub use simple_vector2d::Vector2;

#[derive(Default, Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct BasicObject {
    pub position: Vect,
    pub velocity: Vect,
}

#[derive(Default, Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct RotatedPos {
    pub pos: Vect,
    pub rotation: u8
}

use std::f32::consts::PI;

pub const TAU: f32 = 2. * PI;

impl<'a> From<&'a RotatableObject> for RotatedPos {
    fn from(sp: &'a RotatableObject) -> Self {
        RotatedPos {
            pos: sp.position,
            rotation: (sp.rotation * 256. / TAU) as u8
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct RotatableObject {
    pub position: Vect,
    pub velocity: Vect,
    pub rotation: f32
}

impl BasicObject {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        BasicObject {
            position: Vector2(x, y),
            velocity: Vector2(vx, vy),
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct Planet {
    pub obj: BasicObject,
    pub health: u8
}

#[derive(Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct Player{
    pub obj: RotatableObject,
    pub health: u8
}

impl Default for Player {
    fn default() -> Self {
        Player {
            obj: Default::default(),
            health: 5
        }
    }
}

impl Planet {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Planet {
            obj: BasicObject::new(x, y, vx, vy),
            health: 5
        }
    }
}

use rand::{Rand, Rng};

impl Rand for Planet {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let v = (rng.gen_range(-100., 100.), rng.gen_range(-100., 100.));
        Planet::new(rng.gen_range(-W, W), rng.gen_range(-H, H), v.0, v.1)
    }
}

impl<'a> From<&'a Player> for RotatedPos {
    fn from(p: &'a Player) -> Self {
        From::from(&p.obj)
    }
}

const W: f32 = 1200./2.;
const H: f32 =  900./2.;

/// Wraps `p` if out of bounds
pub fn stay_in_bounds(p: &mut Vect) {
    if p.0 < -W {
        p.0 += 2. * W;
    }
    if p.0 > W {
        p.0 -= 2. * W;
    }
    if p.1 < -H {
        p.1 += 2. * H;
    }
    if p.1 > H {
        p.1 -= 2. * H;
    }
}
