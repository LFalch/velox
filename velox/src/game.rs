use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

use velox_core::obj::{BasicObject, RotatableObject, stay_in_bounds};
use velox_core::net::*;

use piston_window::*;

macro_rules! assets {
    ($base:ident; $($tex:ident),*) => {
        struct $base {
            $($tex: G2dTexture,)*
        }

        impl $base {
            fn new(w: &mut PistonWindow) -> Self {
                let ts = TextureSettings::new();
                $(
                let $tex = Texture::from_path(&mut w.factory,
                    concat!("tex/", stringify!($tex), ".png"), Flip::None, &ts).unwrap();
                )*
                $base {
                    $($tex: $tex,)*
                }
            }
        }
    };
}

assets!{Assets;
    // arrow,
    // sun,
    laser,
    planet,
    ship
}

pub struct SpaceShooter {
    window: PistonWindow,
    assets: Assets,
    socket: Arc<ClientSocket>,
    planets: Arc<Mutex<BTreeMap<Idx, BasicObject>>>,
    players: Arc<Mutex<BTreeMap<Idx, RotatableObject>>>,
    lasers: Arc<Mutex<BTreeMap<Idx, RotatableObject>>>,
    health: Arc<Mutex<u8>>,
}

impl SpaceShooter {
    pub fn new(mut window: PistonWindow, server: &str) -> Self {
        SpaceShooter {
            assets: Assets::new(&mut window),
            window: window,
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
    pub fn run(self) {
        let SpaceShooter {
            assets,
            mut window,
            planets,
            players,
            lasers,
            health,
            socket,
        } = self;
        let (mut up, mut down, mut left, mut right)
            = (false, false, false, false);

        while let Some(e) = window.next() {
            match e {
                Input::Release(b) => {
                    match b {
                        Button::Keyboard(Key::Space) => {socket.send(ClientPacket::Shoot).unwrap();},
                        Button::Keyboard(Key::J) => {
                            println!("Planets: {:#?}", *planets.lock().unwrap());
                            println!("Players: {:#?}", *players.lock().unwrap());
                            println!("Lasers: {:#?}", *lasers.lock().unwrap());
                        }
                        Button::Keyboard(Key::Up) | Button::Keyboard(Key::W) => up = false,
                        Button::Keyboard(Key::Down) | Button::Keyboard(Key::S) => down = false,
                        Button::Keyboard(Key::Left) | Button::Keyboard(Key::A) => left = false,
                        Button::Keyboard(Key::Right) | Button::Keyboard(Key::D) => right = false,
                        _ => ()
                    }
                }
                Input::Press(b) => {
                    match b {
                        Button::Keyboard(Key::Up) | Button::Keyboard(Key::W) => up = true,
                        Button::Keyboard(Key::Down) | Button::Keyboard(Key::S) => down = true,
                        Button::Keyboard(Key::Left) | Button::Keyboard(Key::A) => left = true,
                        Button::Keyboard(Key::Right) | Button::Keyboard(Key::D) => right = true,
                        _ => ()
                    }
                }
                Input::Render(r) => {
                    let w = r.width as f64;
                    let h = r.height as f64;
                    window.draw_2d(&e, |c, g| {
                        clear([0., 0., 0., 1.], g);

                        for planet in planets.lock().unwrap().values() {
                            let (x, y) = planet.position.into();
                            image(&assets.planet, c.transform.trans(x as f64+w/2., y as f64+h/2.).trans(-32., -32.), g)
                        }

                        for player in players.lock().unwrap().values() {
                            let (x, y) = player.position.into();
                            let mat = [[1., 0., 0.], [0., 1., 0.]].trans(x as f64+w/2., y as f64+h/2.).rot_rad(player.rotation as f64).trans(-16., -16.);
                            image(&assets.ship, c.transform.append_transform(mat), g)
                        }

                        for laser in lasers.lock().unwrap().values() {
                            let (x, y) = laser.position.into();
                            let mat = [[1., 0., 0.], [0., 1., 0.]].trans(x as f64+w/2., y as f64+h/2.).rot_rad(laser.rotation as f64).trans(-16., -16.);
                            image(&assets.laser, c.transform.append_transform(mat), g)
                        }

                        let hp = *health.lock().unwrap();
                        rectangle([0.77, 0.77, 0.77, 0.6], [0., 0., 160., 40.], c.transform, g);
                        rectangle([0., 1., 0., 0.6], [10., 5., 30.*hp as f64, 20.], c.transform, g);
                    });
                }
                Input::Update(u) => {
                    let (mut impulse, mut rotation) = (0., 0.);
                    if up {
                        impulse += 1.;
                    }
                    if down {
                        impulse -= 1.;
                    }
                    if right {
                        rotation += 1.;
                    }
                    if left {
                        rotation -= 1.;
                    }
                    if impulse != 0. {
                        socket.send(ClientPacket::PlayerImpulse(impulse * 400. * u.dt as f32)).unwrap();
                    }
                    if rotation != 0. {
                        socket.send(ClientPacket::PlayerRotate(rotation * 2. * u.dt as f32)).unwrap();
                    }

                    for planet in planets.lock().unwrap().values_mut() {
                        planet.position += planet.velocity * u.dt as f32;
                        stay_in_bounds(&mut planet.position);
                    }

                    for player in players.lock().unwrap().values_mut() {
                        player.position += player.velocity * u.dt as f32;
                        stay_in_bounds(&mut player.position);
                    }

                    for laser in lasers.lock().unwrap().values_mut() {
                        laser.position += laser.velocity * u.dt as f32;
                        stay_in_bounds(&mut laser.position);
                    }
                }
                Input::Close(_) => {socket.send(ClientPacket::Disconnect).unwrap();}
                _ => {} // Catch uninteresting events
            }
        }
    }
}

// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
