use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;
use std::thread;

use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate, Graphics};
use simple_vector2d::Vector2;
use bincode::rustc_serialize::{encode, decode};
use bincode::SizeLimit;

pub type Vect = Vector2<f32>;

mod phys;
pub mod serv;

#[derive(Default)]
struct TextureBase(HashMap<String, Texture>);

fn create_texture(graphics: &Graphics, name: &str) -> Texture {
    match Texture::from_file(graphics, format!("tex/{}.png", name)) {
        Ok(t) => t,
        Err(_) => panic!("Failed to load texture {}", name),
    }
}

impl TextureBase {
    fn load<S: ToString>(&mut self, graphics: &Graphics, name: S) {
        let s = name.to_string();
        self.0.insert(name.to_string(), create_texture(graphics, &s));
    }
    fn get_tex(&self, name: &str) -> &Texture {
        if let Some(t) = self.0.get(name) {
            t
        } else {
            panic!("Texture {} not loaded", name)
        }
    }
}

pub struct SpaceShooter {
    texture_base: TextureBase,
    planets: Arc<Mutex<Vec<Vect>>>,
    players: Arc<Mutex<Vec<phys::RotatedPos>>>,
    lasers: Arc<Mutex<Vec<phys::RotatedPos>>>,
    socket: Arc<UdpSocket>,
}

impl SpaceShooter {
    pub fn new(graphics: &Graphics, server: &str) -> Self {
        SpaceShooter {
            texture_base: {
                let mut tb = TextureBase::default();
                tb.load(graphics, "planet");
                tb.load(graphics, "ship");
                tb.load(graphics, "laser");
                tb
            },
            planets: Default::default(),
            players: Default::default(),
            lasers: Default::default(),
            socket: {
                let s = UdpSocket::bind(("0.0.0.0:0")).unwrap();
                s.connect(server).unwrap();
                s.send(&encode(&ClientPacket::Connect, SizeLimit::Infinite).unwrap()).unwrap();
                if let Ok(addr) = s.local_addr() {
                    println!("Bound to {}", addr);
                }
                Arc::new(s)
            }
        }
    }
    // YORO
    pub fn start_network_thread(&self) {
        let socket = self.socket.clone();
        let planets_m = self.planets.clone();
        let players_m = self.players.clone();
        let lasers_m = self.lasers.clone();
        thread::spawn(move || {
            loop {
                let mut buf = [0u8; 1024];
                match socket.recv(&mut buf) {
                    Ok(size) => {
                        match decode(&buf[..size]).unwrap() {
                            ServerPacket::Update{planets, players, lasers} => {
                                *planets_m.lock().unwrap() = planets;
                                *players_m.lock().unwrap() = players;
                                * lasers_m.lock().unwrap() = lasers;
                            }
                            ServerPacket::DisconnectAck => break
                        }
                    },
                    Err(_) => {
                        send_packet(&socket, ClientPacket::Error);
                    }
                }
            }
            println!("Network thread successfully stopped");
        });
    }
}

impl Drop for SpaceShooter {
    fn drop(&mut self) {
        self.socket.send(&encode(&ClientPacket::Disconnect, SizeLimit::Infinite).unwrap()).unwrap();
    }
}

use self::serv::{ClientPacket, ServerPacket};

fn send_packet(socket: &UdpSocket, packet: ClientPacket) {
    encode(&packet, SizeLimit::Infinite)
        .map(|d| socket.send(&d).unwrap()).unwrap();
}

impl Game for SpaceShooter {
    type ReturnType = GameUpdate;
    fn frame(&mut self, info: &FrameInfo, drawer: &mut Drawer) -> GameUpdate {
        when!{info;
            false, Escape => {
                return GameUpdate::Close
            },
            false, Space => {
                send_packet(&self.socket, ClientPacket::Shoot)
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
            send_packet(&self.socket, ClientPacket::PlayerImpulse(impulse * 400. * info.delta))
        }
        if rotation != 0. {
            send_packet(&self.socket, ClientPacket::PlayerRotate(rotation * 2. * info.delta))
        }

        let planet_tex = self.texture_base.get_tex("planet");
        let player_tex = self.texture_base.get_tex("ship");
        let laser_tex = self.texture_base.get_tex("laser");

        drawer.clear(0., 0., 0.);

        for &planet in self.planets.lock().unwrap().iter() {
            planet_tex.drawer()
            .pos(planet.into())
            .draw(drawer);
        }

        for &player in self.players.lock().unwrap().iter() {
            player_tex.drawer()
            .pos(player.pos.into())
            .rotation(player.rotation as f32 / 256. * phys::TAU)
            .draw(drawer);
        }

        for &laser in self.lasers.lock().unwrap().iter() {
            laser_tex.drawer()
            .pos(laser.pos.into())
            .rotation(laser.rotation as f32 / 256. * phys::TAU)
            .draw(drawer);
        }

        GameUpdate::Nothing
    }
}
// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
