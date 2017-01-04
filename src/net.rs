use super::obj::{Vect, RotatedPos};

use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::io::Error;

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

const BUFFER_SIZE: usize = 1024;

pub struct ClientSocket(UdpSocket);

impl ClientSocket {
    pub fn new<S: ToSocketAddrs>(server: S) -> Self {
        let s = UdpSocket::bind("0.0.0.0:0").unwrap();
        s.connect(server).unwrap();
        s.send(&encode(&ClientPacket::Connect, SizeLimit::Infinite).unwrap()).unwrap();
        if let Ok(addr) = s.local_addr() {
            println!("Bound to {}", addr);
        }
        ClientSocket(s)
    }
    pub fn recv(&self) -> Result<ServerPacket, Error> {
        let mut buf = [0u8; BUFFER_SIZE];
        self.0.recv(&mut buf)
            .map(|size| decode(&buf[..size]).unwrap())
    }
    pub fn send(&self, packet: ClientPacket) -> Result<usize, Error> {
        let d = encode(&packet, SizeLimit::Infinite).unwrap();
        self.0.send(&d)
    }
}

/* Finish `ServerSocket`
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type PlayersAmhm<T> = Arc<Mutex<HashMap<SocketAddr, T>>>;

pub struct ServerSocket<T>{
    socket: UdpSocket,
    players: PlayersAmhm<T>,
}

impl<T> ServerSocket<T> {
    pub fn new<S: ToSocketAddrs>(bind_addr: S, players: PlayersAmhm<T>) -> Self {
        ServerSocket{
            socket: UdpSocket::bind(bind_addr).unwrap(),
            players: players
        }
    }
    pub fn recv(&self) -> Result<ClientPacket, Error> {
        let mut buf = [0u8; 20];
        self.0.recv(&mut buf)
            .map(|size| decode(&buf[..size]).unwrap())
    }
    pub fn send(&self, packet: ServerPacket) -> Result<usize, Error> {
        let d = encode(&packet, SizeLimit::Infinite).unwrap();
        self.0.send(&d)
    }
}
*/
