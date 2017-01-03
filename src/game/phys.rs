use super::Vect;
use simple_vector2d::Vector2;

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
            pos: sp.obj.position,
            rotation: (((sp.rotation + TAU) % TAU) * 256. / TAU) as u8
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct RotatableObject {
    pub obj: BasicObject,
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
