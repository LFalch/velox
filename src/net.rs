use super::obj::{Vect, RotatedPos};

use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::io::Error;

pub use bincode::rustc_serialize::{encode, decode};
pub use bincode::SizeLimit;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub enum ClientPacket {
    Connect,
    PlayerImpulse(f32),
    PlayerRotate(f32),
    Shoot,
    Disconnect,
    Error
}

#[derive(Default, RustcEncodable, RustcDecodable, Debug)]
pub struct ObjectsUpdate {
    pub players: Vec<RotatedPos>,
    pub lasers: Vec<RotatedPos>,
    pub planets: Vec<Vect>
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
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

pub struct ServerSocket(UdpSocket);

impl ServerSocket {
    pub fn new<S: ToSocketAddrs>(bind_addr: S) -> Self {
        ServerSocket(UdpSocket::bind(bind_addr).unwrap())
    }
    pub fn recv(&self) -> Option<(SocketAddr, ClientPacket)> {
        let mut buf = [0u8; 20];
        let ret = match self.0.recv_from(&mut buf) {
            Ok((size, remote)) => decode(&buf[..size]).ok().map(|p| (remote, p)),
            Err(_) => None,
        };
        ret
    }
    pub fn send_all<'a, I: 'a>(&self, packet: ServerPacket, addrs: I) -> Result<(), Error>
    where I: IntoIterator<Item=&'a SocketAddr> {
        let data = encode(&packet, SizeLimit::Infinite).unwrap();
        for addr in addrs {
            self.0.send_to(&data, addr)?;
        }
        Ok(())
    }
    pub fn send(&self, packet: ServerPacket, addr: &SocketAddr) -> Result<usize, Error> {
        let data = encode(&packet, SizeLimit::Infinite).unwrap();
        self.0.send_to(&data, addr)
    }
}
