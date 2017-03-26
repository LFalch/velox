use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate, Graphics, Quad};

use velox_core::obj::{BasicObject, RotatableObject, stay_in_bounds};
use velox_core::net::*;

macro_rules! assets {
    ($base:ident, $g:ident; $($tex:ident),*; $($quad:ident : $def:expr,)*) => {
        struct $base {
            $($tex: Texture,)*
            $($quad: Quad,)*
        }

        impl $base {
            fn new($g: &Graphics) -> Self {
                $(
                let $tex = Texture::from_png_bytes($g, include_bytes!(concat!("../tex/", stringify!($tex), ".png"))).unwrap();
                )*
                $base {
                    $($tex: $tex,)*
                    $($quad: $def,)*
                }
            }
        }
    };
}

assets!{Assets, graphics;
    // arrow,
    // sun,
    laser,
    planet,
    ship;
    hp_bar: Quad::new_rect(graphics, [0.77, 0.77, 0.77, 0.6], 160., 40.).unwrap(),
    hp: Quad::new_rect(graphics, [0., 1., 0., 0.6], 30., 30.).unwrap(),
}

pub struct SpaceShooter {
    assets: Assets,
    socket: Arc<ClientSocket>,
    planets: Arc<Mutex<BTreeMap<Idx, BasicObject>>>,
    players: Arc<Mutex<BTreeMap<Idx, RotatableObject>>>,
    lasers: Arc<Mutex<BTreeMap<Idx, RotatableObject>>>,
    health: Arc<Mutex<u8>>,
}

impl SpaceShooter {
    pub fn new(graphics: &Graphics, server: &str) -> Self {
        SpaceShooter {
            assets: Assets::new(graphics),
            planets: Arc::default(),
            players: Arc::default(),
            lasers: Arc::default(),
            health: Arc::default(),
            socket: Arc::new(ClientSocket::new(server))
        }
    }
    // YORO
    pub fn start_network_thread(&self) {
        let socket = self.socket.clone();
        let lasers_m = self.lasers.clone();
        let planets_m = self.planets.clone();
        let players_m = self.players.clone();
        let health_m = self.health.clone();
        thread::spawn(move || {
            loop {
                let p = socket.recv();
                match p {
                    Ok(ServerPacket::PlayersAndPlanets {
                        players,
                        planets
                    }) => {
                        *planets_m.lock().unwrap() = planets;
                        *players_m.lock().unwrap() = players;
                    }
                    Ok(ServerPacket::Lasers(mut lasers)) => {
                        lasers_m.lock().unwrap().append(&mut lasers);
                    }
                    Ok(ServerPacket::UpdateLaser(i, l)) => {
                        lasers_m.lock().unwrap().insert(i, l);
                    }
                    Ok(ServerPacket::UpdatePlanet(i, p)) => {
                        planets_m.lock().unwrap().insert(i, p);
                    }
                    Ok(ServerPacket::UpdatePlayer(i, p)) => {
                        players_m.lock().unwrap().insert(i, p);
                    }
                    Ok(ServerPacket::DeletePlayer(player_id)) => {
                        players_m.lock().unwrap().remove(&player_id);
                    }
                    Ok(ServerPacket::DeletePlanets(ps)) => {
                        let mut planets = planets_m.lock().unwrap();
                        for i in ps.into_iter() {
                            planets.remove(&i);
                        }
                    }
                    Ok(ServerPacket::DeleteLasers(ls)) => {
                        let mut lasers = lasers_m.lock().unwrap();
                        for i in ls.into_iter() {
                            lasers.remove(&i);
                        }
                    }
                    Ok(ServerPacket::UpdateHealth(h)) => {
                        *health_m.lock().unwrap() = h;
                    }
                    Ok(ServerPacket::DisconnectAck) => break,
                    Err(e) => println!("Error! {:?}", e),
                }
            }
            println!("Network thread successfully stopped");
        });
    }
}

impl Drop for SpaceShooter {
    fn drop(&mut self) {
        self.socket.send(ClientPacket::Disconnect).unwrap();
    }
}

impl Game for SpaceShooter {
    type ReturnType = GameUpdate;
    fn frame(&mut self, info: &FrameInfo, drawer: &mut Drawer) -> GameUpdate {
        when!{info;
            false, Escape => {
                return GameUpdate::Close
            },
            false, Space => {
                self.socket.send(ClientPacket::Shoot).unwrap();
            },
            false, J => {
                println!("Planets: {:#?}", *self.planets.lock().unwrap());
                println!("Players: {:#?}", *self.players.lock().unwrap());
                println!("Lasers: {:#?}", *self.lasers.lock().unwrap());
            }
        }
        let mut impulse = 0.;
        let mut rotation = 0.;
        is_down! {info;
            W, Up => {
                impulse += 1.;
            },
            S, Down => {
                impulse -= 1.;
            },
            D, Right => {
                rotation -= 1.;
            },
            A, Left => {
                rotation += 1.;
            }
        }
        if impulse != 0. {
            self.socket.send(ClientPacket::PlayerImpulse(impulse * 400. * info.delta)).unwrap();
        }
        if rotation != 0. {
            self.socket.send(ClientPacket::PlayerRotate(rotation * 2. * info.delta)).unwrap();
        }

        drawer.clear(0., 0., 0.);

        for planet in self.planets.lock().unwrap().values_mut() {
            planet.position += planet.velocity * info.delta;
            stay_in_bounds(&mut planet.position);

            self.assets.planet.drawer()
            .pos(planet.position.into())
            .draw(drawer);
        }

        for player in self.players.lock().unwrap().values_mut() {
            player.position += player.velocity * info.delta;
            stay_in_bounds(&mut player.position);

            self.assets.ship.drawer()
            .pos(player.position.into())
            .rotation(player.rotation)
            .draw(drawer);
        }

        for laser in self.lasers.lock().unwrap().values_mut() {
            laser.position += laser.velocity * info.delta;
            stay_in_bounds(&mut laser.position);

            self.assets.laser.drawer()
            .pos(laser.position.into())
            .rotation(laser.rotation)
            .draw(drawer);
        }

        let hp = *self.health.lock().unwrap();
        self.assets.hp_bar.drawer()
            .pos((-500., 430.))
            .draw(drawer);
        for i in 0..hp {
            self.assets.hp.drawer()
                .pos((-560.+30.*i as f32, 430.))
                .draw(drawer);
        }

        GameUpdate::Nothing
    }
}
// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
