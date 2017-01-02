use simple_vector2d::Vector2;
use bincode::rustc_serialize::{encode, decode};
use bincode::SizeLimit;

use std::net::{UdpSocket, Ipv4Addr, SocketAddr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::thread;

use super::phys::BasicObject;
use super::Vect;

pub struct Server {
    planets: Arc<Mutex<Vec<BasicObject>>>,
    server_socket: Arc<UdpSocket>,
    players: Arc<Mutex<HashSet<SocketAddr>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: Arc::default(),
            players: Arc::default(),
            server_socket: Arc::new(UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 7351)).unwrap()),
        }
    }
    pub fn update(&self, delta: f32) {
        let mut planets = self.planets.lock().unwrap();
        let others: Vec<_> = planets.iter().cloned().collect();

        for (i, planet) in planets.iter_mut().enumerate() {
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
    }
    pub fn run(self) {
        let listener_server_socket = self.server_socket.clone();
        let listener_planets = self.planets.clone();
        let listener_players = self.players.clone();

        let _listener = thread::spawn(move || {
            loop {
                let mut buf = [0u8; 16];
                match listener_server_socket.recv_from(&mut buf) {
                    Ok((_, remote)) => {
                        listener_players.lock().unwrap().insert(remote);
                    }
                    Err(e) => {
                        println!("Error receiving packet! {:?}", e);
                        continue
                    }
                }

                let new_planet: BasicObject = match decode(&buf) {
                    Ok(p) => p,
                    Err(_) => {
                        println!("Decoding error");
                        continue
                    }
                };
                let mut planets = listener_planets.lock().unwrap();
                planets.push(new_planet);
            }
        });

        let mut last_time = Instant::now();
        loop {
            let now = Instant::now();
            let dur = now-last_time;
            last_time = now;
            self.update(dur.as_secs() as f32+ 1e-9 * dur.subsec_nanos() as f32);
            let planets_data = encode(&*self.planets.lock().unwrap(), SizeLimit::Infinite).unwrap();
            for addr in self.players.lock().unwrap().iter() {
                println!("Sent packet: {:?}", self.server_socket.send_to(&planets_data, addr));
            }
            thread::sleep(Duration::from_millis(20));
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
