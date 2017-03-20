use super::obj::{BasicObject, RotatableObject};

use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::collections::BTreeMap;
use std::io::Error;

use bincode::{serialize, deserialize, Bounded};
pub use bincode::serialized_size;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientPacket {
    Connect,
    PlayerImpulse(f32),
    PlayerRotate(f32),
    Shoot,
    Disconnect,
}

pub type Idx = u16;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    PlayersAndPlanets {
        players: BTreeMap<Idx, RotatableObject>,
        planets: BTreeMap<Idx, BasicObject>
    },
    Lasers(BTreeMap<Idx, RotatableObject>),
    UpdatePlayer(Idx, RotatableObject),
    UpdateLaser(Idx, RotatableObject),
    UpdatePlanet(Idx, BasicObject),
    DeletePlayer(Idx),
    DeleteLasers(Vec<Idx>),
    DeletePlanets(Vec<Idx>),
    DisconnectAck
}

fn invalid_data_error<E>(e: E) -> Error
where E: Into<Box<::std::error::Error + Send + Sync>>{
    Error::new(::std::io::ErrorKind::InvalidData, e)
}

const BUFFER_SIZE: usize = 1024;
const BUFFER_SIZE64: u64 = BUFFER_SIZE as u64;
const BUFFER_SIZE_SRV: usize = 20;
const BUFFER_SIZE_SRV64: u64 = BUFFER_SIZE_SRV as u64;

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
            .and_then(|size| deserialize(&buf[..size]).map_err(invalid_data_error))
    }
    pub fn send(&self, packet: ClientPacket) -> Result<usize, Error> {
        let d = serialize(&packet, Bounded(BUFFER_SIZE_SRV64)).map_err(invalid_data_error)?;
        self.0.send(&d)
    }
}

pub struct ServerSocket(UdpSocket);

impl ServerSocket {
    pub fn new<S: ToSocketAddrs>(bind_addr: S) -> Self {
        ServerSocket(UdpSocket::bind(bind_addr).unwrap())
    }
    pub fn recv(&self) -> Result<(SocketAddr, ClientPacket), Error> {
        let mut buf = [0u8; BUFFER_SIZE_SRV];
        match self.0.recv_from(&mut buf) {
            Ok((size, remote)) => {
                deserialize(&buf[..size]).map(|p| (remote, p)).map_err(invalid_data_error)
            }
            Err(e) => Err(e),
        }
    }
    pub fn send_all<'a, I: 'a>(&self, packet: ServerPacket, addrs: I) -> Result<(), Error>
    where I: IntoIterator<Item=&'a SocketAddr> {
        let data = serialize(&packet, Bounded(BUFFER_SIZE64)).map_err(invalid_data_error)?;
        for addr in addrs {
            self.0.send_to(&data, addr)?;
        }
        Ok(())
    }
    pub fn send(&self, packet: ServerPacket, addr: &SocketAddr) -> Result<usize, Error> {
        let data = serialize(&packet, Bounded(BUFFER_SIZE64)).map_err(invalid_data_error)?;
        self.0.send_to(&data, addr)
    }
}
