extern crate piston_window;
extern crate velox_core;

use velox_core::obj::{Vector2, RotatableObject, Planet, Player};

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
                    concat!("../velox/tex/", stringify!($tex), ".png"), Flip::None, &ts).unwrap();
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

pub fn pos_mat(x: f64, y: f64, width: f64, height: f64, w: f64, h: f64) -> math::Matrix2d {
    [[1., 0., 0.], [0., 1., 0.]].trans(x+w - width, y+h - height)
}

pub fn pos_rot_mat(x: f64, y: f64, width: f64, height: f64, w: f64, h: f64, rot: f64) -> math::Matrix2d {
    [[1., 0., 0.], [0., 1., 0.]].trans(x+w, y+h).rot_rad(rot).trans(-width, -height)
}

fn main() {
    let mut window: PistonWindow =
    WindowSettings::new(format!("Space Sandbox {}", env!("CARGO_PKG_VERSION")), [1200, 900])
            .exit_on_esc(true)
            .vsync(true)
            .build().unwrap();
    let assets = Assets::new(&mut window);
    let mut planets = Vec::<Planet>::new();
    let mut player = Player::default();
    let mut lasers = Vec::<RotatableObject>::new();

    let mut up = false;
    let mut down = false;
    let mut left = false;
    let mut right = false;
    let mut mouse_pos = (0., 0.);
    let mut rw = 600.;
    let mut rh = 450.;

    let mut cr_pos = Vector2::default();
    let mut creating = false;

    while let Some(e) = window.next() {
        match e {
            Event::Input(Input::Button(b)) => {
                let press = b.state == ButtonState::Press;

                match b.button {
                    Button::Keyboard(Key::Space) if press => {
                        let dir = Vector2::unit_vector(player.obj.rotation);

                        let new_laser = RotatableObject::new(player.obj.pos() + 42. * dir,
                            player.obj.vel() + 400. * dir, player.obj.rotation);

                        lasers.push(new_laser);
                    }
                    Button::Keyboard(Key::Up) | Button::Keyboard(Key::W) => up = press,
                    Button::Keyboard(Key::Down) | Button::Keyboard(Key::S) => down = press,
                    Button::Keyboard(Key::Left) | Button::Keyboard(Key::A) => left = press,
                    Button::Keyboard(Key::Right) | Button::Keyboard(Key::D) => right = press,
                    Button::Mouse(MouseButton::Left) => {
                        if press {
                            creating = true;
                            cr_pos = Vector2::from(mouse_pos);
                        } else {
                            creating = false;
                            let vel = cr_pos - Vector2::from(mouse_pos);
                            planets.push(Planet::new(cr_pos.0, cr_pos.1, vel.0, vel.1));
                        }
                    }
                    _ => ()
                }
            }
            Event::Input(Input::Move(Motion::MouseCursor(x, y))) => mouse_pos = (x as f32-rw, y as f32-rh),
            Event::Loop(Loop::Render(r)) => {
                let w = r.width as f64/2.;
                let h = r.height as f64/2.;
                rw = w as f32;
                rh = h as f32;
                window.draw_2d(&e, |c, g| {
                    clear([0., 0., 0., 1.], g);

                    if creating {
                        image(&assets.planet, c.transform.append_transform(pos_mat(
                            cr_pos.0 as f64, cr_pos.1 as f64, 32., 32., w, h)), g);
                    }

                    for planet in planets.iter() {
                        let (x, y) = planet.obj.pos().into();
                        image(&assets.planet, c.transform.append_transform(pos_mat(
                            x as f64, y as f64, 32., 32., w, h)), g)
                    }

                    for player in Some(player) {
                        let (x, y) = player.obj.pos().into();
                        image(&assets.ship, c.transform.append_transform(pos_rot_mat(
                            x as f64, y as f64, 16., 16., w, h, player.obj.rotation as f64)), g)
                    }

                    for laser in lasers.iter() {
                        let (x, y) = laser.pos().into();
                        image(&assets.laser, c.transform.append_transform(pos_rot_mat(
                            x as f64, y as f64, 16., 16., w, h, laser.rotation as f64)), g)
                    }

                    let hp = player.health;
                    rectangle([0.77, 0.77, 0.77, 0.6], [0., 0., 170., 40.], c.transform, g);
                    rectangle([0., 1., 0., 0.6], [10., 5., 30.*hp as f64, 20.], c.transform, g);
                });
            }
            Event::Loop(Loop::Update(u)) => {
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
                player.obj.rotation += rotation * 2. * u.dt as f32;
                player.obj.acceleration = impulse * 150. * Vector2::unit_vector(player.obj.rotation);

                for planet in planets.iter_mut() {
                    planet.obj.update(u.dt as f32);
                    planet.obj.stay_in_bounds();
                }

                for player in Some(&mut player) {
                    player.obj.update(u.dt as f32);
                    player.obj.stay_in_bounds();
                }

                for laser in lasers.iter_mut() {
                    laser.update(u.dt as f32);
                    laser.stay_in_bounds();
                }
            }
            Event::Input(Input::Close(_)) => {}
            _ => {} // Catch uninteresting events
        }
    }
}

// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }
