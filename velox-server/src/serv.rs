use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, BTreeMap};
use std::thread;

use velox_core::net::*;
use velox_core::obj::{Vector2, RotatableObject, Planet, Player};

pub struct Server {
    planets: Arc<Mutex<BTreeMap<Idx, Planet>>>,
    server_socket: Arc<ServerSocket>,
    connections: Arc<Mutex<HashMap<SocketAddr, Idx>>>,
    players: Arc<Mutex<BTreeMap<Idx, Player>>>,
    deads: Vec<SocketAddr>,
    lasers: Arc<Mutex<BTreeMap<Idx, RotatableObject>>>
}

#[inline]
fn remove_player(socket: &ServerSocket, connections: &mut HashMap<SocketAddr, Idx>, players: &mut BTreeMap<Idx, Player>, dead: SocketAddr) {
    socket.send(ServerPacket::DisconnectAck, &dead).unwrap();

    if let Some(dead_id) = connections.remove(&dead) {
        players.remove(&dead_id);
        socket.send_all(ServerPacket::DeletePlayer(dead_id), connections.keys()).unwrap();
    }
}

#[inline]
fn fit_in<T>(elem: T, tree_map: &mut BTreeMap<Idx, T>) -> Idx {
    let idx = (0..).filter(|i| !tree_map.contains_key(i)).next().unwrap();
    let old = tree_map.insert(idx, elem);
    debug_assert!(old.is_none());
    idx
}

impl Server {
    pub fn new() -> Self {
        let planets = vec![
            Planet::new(0., 0., 10., 2.),
            Planet::new(50., 0., -10., 2.),
            Planet::new(0., 0., 10., -2.),
            Planet::new(0., 0., -10., -2.),
            Planet::new(0., 400., 50., -20.),
        ];

        Server {
            planets: Arc::new(Mutex::new((0..).zip(planets.into_iter()).collect())),
            deads: Vec::new(),
            lasers: Arc::default(),
            players: Arc::default(),
            connections: Arc::default(),
            server_socket: Arc::new(ServerSocket::new((Ipv4Addr::new(0, 0, 0, 0), 7351))),
        }
    }
    pub fn update(&mut self, delta: f32) {
        let mut dead_lasers = Vec::new();
        let mut dead_planets = Vec::new();
        let player_addrs: Vec<_> = self.connections.lock().unwrap().keys().cloned().collect();

        for (&i, planet) in self.planets.lock().unwrap().iter_mut() {
            for (&j, laser) in self.lasers.lock().unwrap().iter() {
                if planet.obj.pos().distance_to(laser.pos()) < 32. {
                    dead_lasers.push(j);
                    planet.health = planet.health.saturating_sub(1);
                }
            }

            if planet.health == 0 {
                dead_planets.push(i)
            } else {
                planet.obj.update(delta);

                if planet.obj.stay_in_bounds() {
                    self.server_socket.send_all(ServerPacket::UpdatePlanet(i, planet.obj), player_addrs.iter()).unwrap();
                }
            }
        }
        {
            let mut planets = self.planets.lock().unwrap();
            for i in dead_planets.iter() {
                planets.remove(i);
            }
        }

        if !dead_planets.is_empty() {
            self.server_socket.send_all(ServerPacket::DeletePlanets(dead_planets), player_addrs.iter()).unwrap();
        }

        for (addr, i) in self.connections.lock().unwrap().iter_mut() {
            let mut players = self.players.lock().unwrap();
            let player = players.get_mut(i).unwrap();
            for (&l, laser) in self.lasers.lock().unwrap().iter() {
                if player.obj.pos().distance_to(laser.pos()) < 32. {
                    player.health = player.health.saturating_sub(1);
                    self.server_socket.send(ServerPacket::UpdateHealth(player.health), addr).unwrap();
                    dead_lasers.push(l);
                }
            }
            if player.health == 0 {
                println!("{} died!", addr);
                self.deads.push(addr.clone());
            }

            player.obj.update(delta);
            player.obj.stay_in_bounds();
        }

        dead_lasers.dedup();

        let mut lasers = self.lasers.lock().unwrap();

        if !dead_lasers.is_empty() {
            for l in dead_lasers.iter() {
                lasers.remove(l);
            }
            self.server_socket.send_all(ServerPacket::DeleteLasers(dead_lasers), player_addrs.iter()).unwrap();
        }

        for (&i, laser) in lasers.iter_mut() {
            laser.update(delta);

            if laser.stay_in_bounds() {
                self.server_socket.send_all(ServerPacket::UpdateLaser(i, *laser), player_addrs.iter()).unwrap();
            }
        }
        if let Some(dead) = self.deads.pop() {
            let mut players = self.players.lock().unwrap();
            let mut connections = self.connections.lock().unwrap();
            remove_player(&self.server_socket, &mut connections, &mut players, dead);
        }
    }
    pub fn run(mut self) {
        let listener_server_socket = self.server_socket.clone();
        let listener_players = self.players.clone();
        let listener_connections = self.connections.clone();
        let listener_lasers = self.lasers.clone();
        let listener_planets = self.planets.clone();

        let _listener = thread::spawn(move || {
            loop {
                let (remote, packet) = listener_server_socket.recv().unwrap();
                let mut players = listener_players.lock().unwrap();
                let mut connections = listener_connections.lock().unwrap();
                let mut to_send = None;
                match packet {
                    ClientPacket::Connect => {
                        listener_server_socket.send(ServerPacket::PlayersAndPlanets {
                            planets: listener_planets.lock().unwrap().iter().map(|(&i, p)| (i, p.obj)).collect(),
                            players: players.iter().map(|(&i, p)| (i, p.obj)).collect()
                        }, &remote).unwrap();
                        listener_server_socket.send(ServerPacket::UpdateHealth(5), &remote).unwrap();
                        let mut lasers: Vec<_> = listener_lasers.lock().unwrap().iter()
                            .map(|(&i, &l)| (i, l)).collect();
                        while !lasers.is_empty() {
                            let start = lasers.len().saturating_sub(46);
                            let to_send = lasers.drain(start..).collect();
                            listener_server_socket.send(ServerPacket::Lasers(to_send),
                                                        &remote).unwrap();
                        }
                        println!("{} connected!", remote);

                        let idx = fit_in(Player::default(), &mut players);
                        connections.insert(remote, idx);
                        to_send = Some(ServerPacket::UpdatePlayer(idx, RotatableObject::default()));
                    }
                    ClientPacket::PlayerImpulse(a) => {
                        if let Some(i) = connections.get(&remote) {
                            let player = players.get_mut(i).unwrap();
                            player.obj.acceleration = a * Vector2::unit_vector(player.obj.rotation);
                            to_send = Some(ServerPacket::UpdatePlayer(*i, player.obj));
                        }
                    }
                    ClientPacket::PlayerRotate(r) => {
                        if let Some(i) = connections.get(&remote) {
                            let player = players.get_mut(i).unwrap();
                            player.obj.rotation += r;
                            to_send = Some(ServerPacket::UpdatePlayer(*i, player.obj));
                        }
                    }
                    ClientPacket::Shoot => {
                        if let Some(i) = connections.get(&remote) {
                            let player = players[i].obj;
                            let dir = Vector2::unit_vector(player.rotation);

                            let new_laser = RotatableObject::new(player.pos() + 42. * dir,
                                player.vel() + 400. * dir, player.rotation);

                            let mut lasers = listener_lasers.lock().unwrap();
                            let idx = fit_in(new_laser, &mut lasers);
                            to_send = Some(ServerPacket::UpdateLaser(idx, new_laser));
                        }
                    }
                    ClientPacket::Disconnect => {
                        remove_player(&listener_server_socket, &mut connections, &mut players, remote);
                    }
                }

                if let Some(packet) = to_send {
                    listener_server_socket.send_all(packet, connections.keys()).unwrap();
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
                    aggregate_time -= spawn_time;
                    let new_planet: Planet = ::rand::random();
                    let mut planets = self.planets.lock().unwrap();
                    let idx = fit_in(new_planet, &mut planets);
                    self.server_socket.send_all(
                        ServerPacket::UpdatePlanet(idx, new_planet.obj),
                        self.connections.lock().unwrap().keys()
                    ).unwrap();
                }
            }

            self.update(dur.as_secs() as f32 + 1e-9 * dur.subsec_nanos() as f32);
            thread::sleep(Duration::from_millis(18));
        }
        // listener.join();
    }
}
