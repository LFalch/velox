use std::collections::HashMap;
use std::net::UdpSocket;

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
    planets: Vec<Vect>,
    players: Vec<phys::RotatedPos>,
    lasers: Vec<phys::RotatedPos>,
    socket: UdpSocket
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
                s
            }
        }
    }
}

impl Drop for SpaceShooter {
    fn drop(&mut self) {
        self.socket.send(&encode(&ClientPacket::Disconnect, SizeLimit::Infinite).unwrap()).unwrap();
    }
}

use self::serv::{ClientPacket, ServerPacket};

impl SpaceShooter {
    fn send_packet(&self, packet: ClientPacket) {
        encode(&packet, SizeLimit::Infinite)
            .map(|d| self.socket.send(&d).unwrap()).unwrap();
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
                self.send_packet(ClientPacket::Shoot)
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
            self.send_packet(ClientPacket::PlayerImpulse(impulse * 400. * info.delta))
        }
        if rotation != 0. {
            self.send_packet(ClientPacket::PlayerRotate(rotation * 2. * info.delta))
        }

        let planet_tex = self.texture_base.get_tex("planet");
        let player_tex = self.texture_base.get_tex("ship");
        let laser_tex = self.texture_base.get_tex("laser");

        drawer.clear(0., 0., 0.);

        let mut buf = [0u8; 1024];
        let size = self.socket.recv(&mut buf).unwrap();

        match decode(&buf[..size]).unwrap() {
            ServerPacket::Update{planets, players, lasers} => {
                self.planets = planets;
                self.players = players;
                self.lasers = lasers;
            }
        }

        for &planet in &self.planets {
            planet_tex.drawer()
            .pos(planet.into())
            .draw(drawer);
        }

        for &player in &self.players {
            player_tex.drawer()
            .pos(player.pos.into())
            .rotation(player.rotation)
            .draw(drawer);
        }

        for &laser in &self.lasers {
            laser_tex.drawer()
            .pos(laser.pos.into())
            .rotation(laser.rotation)
            .draw(drawer);
        }

        GameUpdate::Nothing
    }
}
// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
