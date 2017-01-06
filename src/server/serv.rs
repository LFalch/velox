use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

use velox::net::*;
use velox::obj::{Vector2, RotatableObject, Planet, Player, stay_in_bounds};

pub struct Server {
    planets: Arc<Mutex<Vec<Planet>>>,
    server_socket: Arc<ServerSocket>,
    players: Arc<Mutex<HashMap<SocketAddr, (usize, Player)>>>,
    deads: Vec<SocketAddr>,
    lasers: Arc<Mutex<Vec<RotatableObject>>>
}

fn remove_player(socket: &ServerSocket, players: &mut HashMap<SocketAddr, (usize, Player)>, dead: SocketAddr) {
    socket.send(ServerPacket::DisconnectAck, &dead).unwrap();

    if let Some((dead_id, _)) = players.remove(&dead) {
        socket.send_all(ServerPacket::DeletePlayer(dead_id), players.keys()).unwrap();
        for &mut (ref mut id, _) in players.values_mut() {
            if *id > dead_id {
                *id -= 1;
            }
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: Arc::new(Mutex::new(vec![
                Planet::new(0., 0., 10., 2.),
                Planet::new(50., 0., -10., 2.),
                Planet::new(0., 0., 10., -2.),
                Planet::new(0., 0., -10., -2.),
                Planet::new(0., 400., 50., -20.),
            ])),
            deads: Vec::new(),
            lasers: Arc::default(),
            players: Arc::default(),
            server_socket: Arc::new(ServerSocket::new((Ipv4Addr::new(0, 0, 0, 0), 7351))),
        }
    }
    pub fn update(&mut self, delta: f32) {
        let mut dead_lasers = Vec::new();
        let mut dead_planets = Vec::new();
        let player_addrs: Vec<_> = self.players.lock().unwrap().keys().cloned().collect();

        for (i, planet) in self.planets.lock().unwrap().iter_mut().enumerate() {
            for (j, laser) in self.lasers.lock().unwrap().iter().enumerate() {
                if planet.obj.position.distance_to(laser.position) < 32. {
                    dead_lasers.push(j);
                    planet.health = planet.health.saturating_sub(1);
                }
            }

            if planet.health == 0 {
                dead_planets.push(i)
            } else {
                planet.obj.position += planet.obj.velocity * delta;

                if stay_in_bounds(&mut planet.obj.position) {
                    self.server_socket.send_all(ServerPacket::UpdatePlanet(i, planet.obj), player_addrs.iter()).unwrap();
                }
            }
        }
        self.planets.lock().unwrap().retain(|p| p.health != 0);

        if !dead_planets.is_empty() {
            self.server_socket.send_all(ServerPacket::DeletePlanets(dead_planets), player_addrs.iter()).unwrap();
        }

        for (addr, &mut (_, ref mut player)) in self.players.lock().unwrap().iter_mut() {
            for (l, laser) in self.lasers.lock().unwrap().iter().enumerate() {
                if player.obj.position.distance_to(laser.position) < 32. {
                    player.health = player.health.saturating_sub(1);
                    dead_lasers.push(l);
                }
            }
            if player.health == 0 {
                println!("{} died!", addr);
                self.deads.push(addr.clone());
            }

            player.obj.position += player.obj.velocity * delta;
            stay_in_bounds(&mut player.obj.position);
        }

        dead_lasers.sort();
        dead_lasers.dedup();

        let mut lasers = self.lasers.lock().unwrap();

        if !dead_lasers.is_empty() {
            for &l in dead_lasers.iter().rev() {
                lasers.remove(l);
            }
            self.server_socket.send_all(ServerPacket::DeleteLasers(dead_lasers), player_addrs.iter()).unwrap();
        }

        for (i, laser) in lasers.iter_mut().enumerate() {
            laser.position += laser.velocity * delta;
            if stay_in_bounds(&mut laser.position) {
                self.server_socket.send_all(ServerPacket::UpdateLaser(i, *laser), player_addrs.iter()).unwrap();
            }
        }
        if let Some(dead) = self.deads.pop() {
            let mut players = self.players.lock().unwrap();
            remove_player(&self.server_socket, &mut players, dead);
        }
    }
    pub fn run(mut self) {
        let listener_server_socket = self.server_socket.clone();
        let listener_players = self.players.clone();
        let listener_lasers = self.lasers.clone();
        let listener_planets = self.planets.clone();

        let _listener = thread::spawn(move || {
            loop {
                let (remote, packet) = listener_server_socket.recv().unwrap();
                let mut players = listener_players.lock().unwrap();
                let mut to_send = None;
                match packet {
                    ClientPacket::Connect => {
                        let i = players.len();
                        listener_server_socket.send(ServerPacket::PlayersAndPlanets {
                            planets: listener_planets.lock().unwrap().iter().map(|p| p.obj).collect(),
                            players: players.values().map(|p| p.1.obj).collect()
                        }, &remote).unwrap();
                        let lasers = listener_lasers.lock().unwrap();
                        listener_server_socket.send(ServerPacket::AllLasers(
                            lasers.iter().cloned().take(50).collect()
                        ), &remote).unwrap();
                        if lasers.len() > 50 {
                            println!("Too many lasers to send to connecting player.\n\
                            They might experience death by invisble lasers.");
                        }
                        println!("{} connected!", remote);
                        players.insert(remote, (i, Player::default()));
                        to_send = Some(ServerPacket::UpdatePlayer(i, RotatableObject::default()));
                    }
                    ClientPacket::PlayerImpulse(v) => {
                        if let Some(&mut (i, ref mut player)) = players.get_mut(&remote) {
                            player.obj.velocity += v * Vector2::unit_vector(player.obj.rotation);
                            to_send = Some(ServerPacket::UpdatePlayer(i, player.obj));
                        }
                    }
                    ClientPacket::PlayerRotate(r) => {
                        if let Some(&mut (i, ref mut player)) = players.get_mut(&remote) {
                            player.obj.rotation += r;
                            to_send = Some(ServerPacket::UpdatePlayer(i, player.obj));
                        }
                    }
                    ClientPacket::Shoot => {
                        if let Some(player) = players.get(&remote) {
                            let mut laser = player.1.obj;
                            let dir = Vector2::unit_vector(laser.rotation);
                            laser.velocity += 400. * dir;
                            laser.position += 42. * dir;

                            let mut lasers = listener_lasers.lock().unwrap();
                            to_send = Some(ServerPacket::UpdateLaser(lasers.len(), laser));
                            lasers.push(laser);
                        }
                    }
                    ClientPacket::Disconnect => {
                        remove_player(&listener_server_socket, &mut players, remote);
                    }
                }

                if let Some(packet) = to_send {
                    listener_server_socket.send_all(packet, players.keys()).unwrap();
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

            if self.planets.lock().unwrap().len() < 5 {
                aggregate_time += dur;
                if aggregate_time >= spawn_time {
                    aggregate_time = Duration::new(0, 0);
                    let new_planet: Planet = ::rand::random();
                    let mut planets = self.planets.lock().unwrap();
                    self.server_socket.send_all(
                        ServerPacket::UpdatePlanet(planets.len(), new_planet.obj),
                        self.players.lock().unwrap().keys()
                    ).unwrap();
                    planets.push(new_planet);
                }
            }

            self.update(dur.as_secs() as f32 + 1e-9 * dur.subsec_nanos() as f32);
            thread::sleep(Duration::from_millis(18));
        }
        // listener.join();
    }
}
