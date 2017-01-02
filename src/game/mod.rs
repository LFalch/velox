use std::collections::HashMap;
use std::net::UdpSocket;

use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate, Graphics};
use simple_vector2d::Vector2;
use bincode::rustc_serialize::{encode, decode};
use bincode::SizeLimit;

pub type Vect = Vector2<f32>;

mod phys;
pub mod serv;

use self::phys::BasicObject;

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
    new_planet: Option<Vect>,
    player: BasicObject,
    socket: UdpSocket,
    synced: bool
}

impl SpaceShooter {
    pub fn new(graphics: &Graphics) -> Self {
        SpaceShooter {
            texture_base: {
                let mut tb = TextureBase::default();
                tb.load(graphics, "planet");
                tb.load(graphics, "ship");
                tb
            },
            synced: false,
            new_planet: Default::default(),
            player: Default::default(),
            socket: {
                let s = UdpSocket::bind(("127.0.0.1:0")).unwrap();
                s.connect("127.0.0.1:7351").unwrap();
                s
            }
        }
    }
}

impl Game for SpaceShooter {
    fn frame(&mut self, info: &FrameInfo, drawer: &mut Drawer) -> GameUpdate {
        when!{info;
            false, Escape => {
                return GameUpdate::Close
            }
        }
        when_mouse!{info;
            true, Left => {
                if self.new_planet.is_none(){
                    self.new_planet = Some(info.mousepos.into());
                }
            },
            false, Left => {
                if let Some(pos) = self.new_planet{
                    let new = encode(&BasicObject::new(pos, pos-info.mousepos.into()), SizeLimit::Infinite).unwrap();
                    self.socket.send(&new).unwrap();
                    self.synced = true;
                    self.new_planet = None;
                }
            }
        }

        let planet_tex = self.texture_base.get_tex("planet");

        drawer.clear(0., 0., 0.);

        if self.synced {
            let mut buf = [0u8; 1024];
            let size = self.socket.recv(&mut buf).unwrap();

            let planets: Vec<BasicObject> = decode(&buf[..size]).unwrap();
            for planet in planets.iter() {
                planet_tex.drawer()
                .pos(planet.position.into())
                .draw(drawer);
            }
        }

        if let Some(p) = self.new_planet {
            planet_tex.drawer()
                .pos(p.into())
                .colour([0.5; 4])
                .draw(drawer);
        }

        self.texture_base
            .get_tex("ship")
            .drawer()
            .pos(self.player.position.into())
            .draw(drawer);

        GameUpdate::Nothing
    }
}
// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
