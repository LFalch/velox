use simple_vector2d::Vector2;
use bincode::rustc_serialize::{encode, decode};
use bincode::SizeLimit;

use std::net::{UdpSocket, Ipv4Addr, SocketAddr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

use super::phys::{ServerPlayer, ClientPlayer, BasicObject};
use super::Vect;

pub struct Server {
    planets: Vec<BasicObject>,
    server_socket: Arc<UdpSocket>,
    players: Arc<Mutex<HashMap<SocketAddr, ServerPlayer>>>,
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ClientPacket {
    Connect,
    PlayerImpulse(f32),
    PlayerRotate(f32),
    Disconnect
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ServerPacket {
    Update{
        players: Vec<ClientPlayer>,
        planets: Vec<Vect>
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: vec![
                BasicObject::new(0., 0., 10., 2.),
                BasicObject::new(50., 0., -10., 2.),
                BasicObject::new(0., 0., 10., -2.),
                BasicObject::new(0., 0., -10., -2.),
                BasicObject::new(0., 400., 50., -20.),
            ],
            players: Arc::default(),
            server_socket: Arc::new(UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 7351)).unwrap()),
        }
    }
    pub fn update(&mut self, delta: f32) {
        let others: Vec<_> = self.planets.iter().cloned().collect();

        for (i, planet) in self.planets.iter_mut().enumerate() {
            planet.position += planet.velocity * delta;

            stay_in_bounds(&mut planet.position);

            for (j, &other) in others.iter().enumerate() {
                let dist = planet.position - other.position;
                let half_dist = dist.length() / 2.;

                if i != j && half_dist < 32. {
                    planet.position += Vector2::unit_vector(dist.direction()) * (32. - half_dist);
                }
            }
        }
        for player in self.players.lock().unwrap().values_mut() {
            player.obj.position += player.obj.velocity * delta;
            stay_in_bounds(&mut player.obj.position);
        }
    }
    pub fn run(mut self) {
        let listener_server_socket = self.server_socket.clone();
        let listener_players = self.players.clone();

        let _listener = thread::spawn(move || {
            loop {
                let mut buf = [0u8; 20];
                let (size, remote) = match listener_server_socket.recv_from(&mut buf) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("Error receiving packet! {:?}", e);
                        continue
                    }
                };

                let packet: ClientPacket = match decode(&buf[..size]) {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Decoding error {:?}", e);
                        continue
                    }
                };
                let mut players = listener_players.lock().unwrap();
                match packet {
                    ClientPacket::Connect => {
                        players.insert(remote, ServerPlayer::default());
                    }
                    ClientPacket::PlayerImpulse(v) => {
                        players.get_mut(&remote).map(|b| b.obj.velocity += v * Vector2::unit_vector(b.rotation));
                    }
                    ClientPacket::PlayerRotate(r) => {
                        players.get_mut(&remote).map(|b| b.rotation += r);
                    }
                    ClientPacket::Disconnect => {
                        players.remove(&remote);
                    }
                }
            }
        });

        let mut last_time = Instant::now();
        loop {
            let now = Instant::now();
            let dur = now-last_time;
            last_time = now;
            self.update(dur.as_secs() as f32+ 1e-9 * dur.subsec_nanos() as f32);
            let planets: Vec<_> = self.planets.iter().map(|bo| bo.position).collect();
            let players: Vec<_> = self.players.lock().unwrap().values()
                                              .map(ClientPlayer::from).collect();
            let data = encode(&ServerPacket::Update{planets: planets, players: players}, SizeLimit::Infinite).unwrap();
            for addr in self.players.lock().unwrap().keys() {
                self.server_socket.send_to(&data, addr).unwrap();
            }
            thread::sleep(Duration::from_millis(18));
        }
        // listener.join();
    }
}

const W: f32 = 1200./2.;
const H: f32 =  900./2.;

/// Wraps `p` if out of bounds
fn stay_in_bounds(p: &mut Vect) {
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
