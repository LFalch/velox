use super::obj::{BasicObject, RotatableObject};

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
}

#[derive(Default, RustcEncodable, RustcDecodable, Debug)]
pub struct AllObjects {
    pub players: Vec<RotatableObject>,
    pub lasers: Vec<RotatableObject>,
    pub planets: Vec<BasicObject>
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub enum ServerPacket {
    All(AllObjects),
    UpdatePlayer(usize, RotatableObject),
    UpdateLaser(usize, RotatableObject),
    UpdatePlanet(usize, BasicObject),
    DeletePlayers(Vec<usize>),
    DeleteLasers(Vec<usize>),
    DeletePlanets(Vec<usize>),
    DisconnectAck
}

fn invalid_data_error<E>(e: E) -> Error
where E: Into<Box<::std::error::Error + Send + Sync>>{
    Error::new(::std::io::ErrorKind::InvalidData, e)
}

const BUFFER_SIZE: usize = 1024;

pub struct ClientSocket(UdpSocket);

impl ClientSocket {
    pub fn new<S: ToSocketAddrs>(server: S) -> Self {
        let s = UdpSocket::bind("0.0.0.0:0").unwrap();
        s.connect(server).unwrap();
        let s = ClientSocket(s);
        s.send(ClientPacket::Connect).unwrap();
        if let Ok(addr) = s.0.local_addr() {
            println!("Bound to {}", addr);
        }
        s
    }
    pub fn recv(&self) -> Result<ServerPacket, Error> {
        let mut buf = [0u8; BUFFER_SIZE];
        self.0.recv(&mut buf)
            .and_then(|size| decode(&buf[..size]).map_err(invalid_data_error))
    }
    pub fn send(&self, packet: ClientPacket) -> Result<usize, Error> {
        let d = encode(&packet, SizeLimit::Infinite).map_err(invalid_data_error)?;
        self.0.send(&d)
    }
}

pub struct ServerSocket(UdpSocket);

impl ServerSocket {
    pub fn new<S: ToSocketAddrs>(bind_addr: S) -> Self {
        ServerSocket(UdpSocket::bind(bind_addr).unwrap())
    }
    pub fn recv(&self) -> Result<(SocketAddr, ClientPacket), Error> {
        let mut buf = [0u8; 20];
        match self.0.recv_from(&mut buf) {
            Ok((size, remote)) => {
                decode(&buf[..size]).map(|p| (remote, p)).map_err(invalid_data_error)
            }
            Err(e) => Err(e),
        }
    }
    pub fn send_all<'a, I: 'a>(&self, packet: ServerPacket, addrs: I) -> Result<(), Error>
    where I: IntoIterator<Item=&'a SocketAddr> {
        let data = encode(&packet, SizeLimit::Infinite).map_err(invalid_data_error)?;
        for addr in addrs {
            self.0.send_to(&data, addr)?;
        }
        Ok(())
    }
    pub fn send(&self, packet: ServerPacket, addr: &SocketAddr) -> Result<usize, Error> {
        let data = encode(&packet, SizeLimit::Infinite).map_err(invalid_data_error)?;
        self.0.send_to(&data, addr)
    }
}
