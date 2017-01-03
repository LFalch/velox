use super::obj::{Vect, RotatedPos};

pub use bincode::rustc_serialize::{encode, decode};
pub use bincode::SizeLimit;

#[derive(RustcEncodable, RustcDecodable)]
pub enum ClientPacket {
    Connect,
    PlayerImpulse(f32),
    PlayerRotate(f32),
    Shoot,
    Disconnect,
    Error
}

#[derive(Default, RustcEncodable, RustcDecodable)]
pub struct ObjectsUpdate {
    pub players: Vec<RotatedPos>,
    pub lasers: Vec<RotatedPos>,
    pub planets: Vec<Vect>
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ServerPacket {
    Update(ObjectsUpdate),
    DisconnectAck
}
