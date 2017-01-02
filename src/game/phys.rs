use super::Vect;

#[derive(Default, Debug, Copy, Clone)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct BasicObject {
    pub position: Vect,
    pub velocity: Vect,
}

impl BasicObject {
    pub fn new(position: Vect, velocity: Vect) -> Self {
        BasicObject {
            position: position,
            velocity: velocity,
        }
    }
}
