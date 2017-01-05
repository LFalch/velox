use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs;

use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate, Graphics};

use velox::obj::TAU;
use velox::net::*;

#[derive(Default)]
struct TextureBase(HashMap<String, Texture>);

const TEXTURE_FOLDER: &'static str = "tex";

impl TextureBase {
    fn new(graphics: &Graphics) -> Self {
        let mut hm = HashMap::new();

        for file in fs::read_dir(TEXTURE_FOLDER).expect("read texture folder").filter_map(Result::ok) {
            if file.path().extension().map(|x| x == "png").unwrap_or(false) {
                let file_name = file.file_name();
                let name = file_name.to_str().unwrap()[..file_name.len()-4].to_owned();
                let t = Texture::from_file(graphics, file.path()).unwrap();

                hm.insert(name, t);
            }
        }

        TextureBase(hm)
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
    last_update: Arc<Mutex<ObjectsUpdate>>,
    socket: Arc<ClientSocket>,
}

impl SpaceShooter {
    pub fn new(graphics: &Graphics, server: &str) -> Self {
        SpaceShooter {
            texture_base: TextureBase::new(graphics),
            last_update: Arc::default(),
            socket: Arc::new(ClientSocket::new(server))
        }
    }
    // YORO
    pub fn start_network_thread(&self) {
        let socket = self.socket.clone();
        let update = self.last_update.clone();
        thread::spawn(move || {
            loop {
                match socket.recv() {
                    Ok(ServerPacket::Update(u)) => *update.lock().unwrap() = u,
                    Ok(ServerPacket::DisconnectAck) => break,
                    Err(_) => {
                        socket.send(ClientPacket::Error).unwrap();
                    }
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

        let planet_tex = self.texture_base.get_tex("planet");
        let player_tex = self.texture_base.get_tex("ship");
        let laser_tex = self.texture_base.get_tex("laser");

        drawer.clear(0., 0., 0.);

        let ObjectsUpdate{ref planets, ref players, ref lasers} = *self.last_update.lock().unwrap();

        for &planet in planets.iter() {
            planet_tex.drawer()
            .pos(planet.into())
            .draw(drawer);
        }

        for &player in players.iter() {
            player_tex.drawer()
            .pos(player.pos.into())
            .rotation(player.rotation as f32 / 256. * TAU)
            .draw(drawer);
        }

        for &laser in lasers.iter() {
            laser_tex.drawer()
            .pos(laser.pos.into())
            .rotation(laser.rotation as f32 / 256. * TAU)
            .draw(drawer);
        }

        GameUpdate::Nothing
    }
}
// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
