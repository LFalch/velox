use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

use space_shooter::net::*;
use space_shooter::obj::{Vector2, RotatableObject, RotatedPos, Planet, Player, TAU, stay_in_bounds};

pub struct Server {
    planets: Vec<Planet>,
    server_socket: Arc<ServerSocket>,
    players: Arc<Mutex<HashMap<SocketAddr, Player>>>,
    lasers: Arc<Mutex<Vec<(bool, RotatableObject)>>>
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: vec![
                Planet::new(0., 0., 10., 2.),
                Planet::new(50., 0., -10., 2.),
                Planet::new(0., 0., 10., -2.),
                Planet::new(0., 0., -10., -2.),
                Planet::new(0., 400., 50., -20.),
            ],
            lasers: Arc::default(),
            players: Arc::default(),
            server_socket: Arc::new(ServerSocket::new((Ipv4Addr::new(0, 0, 0, 0), 7351))),
        }
    }
    pub fn update(&mut self, delta: f32) {
        let other_poses: Vec<_> = self.planets.iter().map(|o| o.obj.position).collect();

        for (i, planet) in self.planets.iter_mut().enumerate() {
            for &mut (ref mut alive, ref mut laser) in self.lasers.lock().unwrap().iter_mut() {
                if planet.obj.position.distance_to(laser.position) < 32. {
                    *alive = false;
                    planet.health = planet.health.saturating_sub(1);
                }
            }

            planet.obj.position += planet.obj.velocity * delta;

            stay_in_bounds(&mut planet.obj.position);

            for (j, &other_pos) in other_poses.iter().enumerate() {
                let dist = planet.obj.position - other_pos;
                let half_dist = dist.length() / 2.;

                if i != j && half_dist < 32. {
                    planet.obj.position += Vector2::unit_vector(dist.direction()) * (32. - half_dist);
                }
            }
        }
        self.planets.retain(|p| p.health != 0);
        self.lasers.lock().unwrap().retain(|l| l.0);
        let mut deads = Vec::new();
        for (addr, player) in self.players.lock().unwrap().iter_mut() {
            for &mut (ref mut alive, ref mut laser) in self.lasers.lock().unwrap().iter_mut() {
                if player.obj.position.distance_to(laser.position) < 32. {
                    *alive = false;
                    player.health = player.health.saturating_sub(1);
                    if player.health == 0 {
                        println!("{} died!", addr);
                        deads.push(addr.clone());
                    }
                }
            }

            player.obj.position += player.obj.velocity * delta;
            stay_in_bounds(&mut player.obj.position);
        }
        for laser in self.lasers.lock().unwrap().iter_mut() {
            laser.1.position += laser.1.velocity * delta;
            stay_in_bounds(&mut laser.1.position);
        }
        self.server_socket.send_all(ServerPacket::DisconnectAck, deads.iter()).unwrap();
        for addr in deads {
            self.players.lock().unwrap().remove(&addr);
        }
    }
    pub fn run(mut self) {
        let listener_server_socket = self.server_socket.clone();
        let listener_players = self.players.clone();
        let listener_lasers = self.lasers.clone();

        let _listener = thread::spawn(move || {
            loop {
                let (remote, packet) = listener_server_socket.recv().unwrap();
                let mut players = listener_players.lock().unwrap();
                match packet {
                    ClientPacket::Connect => {
                        players.insert(remote, Player::default());
                    }
                    ClientPacket::PlayerImpulse(v) => {
                        players.get_mut(&remote).map(|b| b.obj.velocity += v * Vector2::unit_vector(b.obj.rotation));
                    }
                    ClientPacket::PlayerRotate(r) => {
                        players.get_mut(&remote).map(|b| b.obj.rotation = (b.obj.rotation + r + TAU) % TAU);
                    }
                    ClientPacket::Shoot => {
                        if let Some(player) = players.get(&remote) {
                            let mut laser = player.obj;
                            let dir = Vector2::unit_vector(laser.rotation);
                            laser.velocity += 400. * dir;
                            laser.position += 42. * dir;
                            listener_lasers.lock().unwrap().push((true, laser));
                        }
                    }
                    ClientPacket::Error => {
                        let mut lasers = listener_lasers.lock().unwrap();
                        for _ in 0..5 {
                            lasers.remove(0);
                        }
                        println!("Laser count {}", lasers.len());
                    }
                    ClientPacket::Disconnect => {
                        players.remove(&remote);
                        listener_server_socket.send(ServerPacket::DisconnectAck, &remote).unwrap();
                    }
                }
            }
        });

        let mut last_time = Instant::now();
        let mut aggregate_time = Duration::new(0, 0);
        let spawn_time = Duration::from_secs(10);

        loop {
            let now = Instant::now();
            let dur = now-last_time;
            last_time = now;

            if self.planets.len() < 5 {
                aggregate_time += dur;
                if aggregate_time >= spawn_time {
                    aggregate_time = Duration::new(0, 0);
                    self.planets.push(::rand::random());
                }
            }

            self.update(dur.as_secs() as f32 + 1e-9 * dur.subsec_nanos() as f32);
            let planets: Vec<_> = self.planets.iter().map(|bo| bo.obj.position).collect();
            let players: Vec<_> = self.players.lock().unwrap().values()
                                              .map(RotatedPos::from).collect();
            let lasers: Vec<_> = self.lasers.lock().unwrap().iter()
                                            .map(|&(_, ref l)| RotatedPos::from(l)).collect();
            self.server_socket.send_all(ServerPacket::Update(ObjectsUpdate {
                planets: planets,
                players: players,
                lasers: lasers
            }), self.players.lock().unwrap().keys()).unwrap();
            thread::sleep(Duration::from_millis(18));
        }
        // listener.join();
    }
}
