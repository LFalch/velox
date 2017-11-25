use std::ops::{Deref, DerefMut};

pub type Vect = Vector2<f32>;
pub use simple_vector2d::Vector2;

#[derive(Default, Debug, Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PhysicsObject {
    position: Vect,
    velocity: Vect,
    pub acceleration: Vect,
}

#[derive(Default, Debug, Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct RotatableObject {
    physics_obj: PhysicsObject,
    pub rotation: f32
}

impl PhysicsObject {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        PhysicsObject {
            position: Vector2(x, y),
            velocity: Vector2(vx, vy),
            acceleration: Vector2(0., 0.),
        }
    }
    pub fn update(&mut self, dt: f32) {
        self.position += 0.5 * self.acceleration * dt * dt + self.velocity * dt;
        self.velocity += self.acceleration * dt;
    }
    #[inline]
    pub fn stay_in_bounds(&mut self) -> bool {
        stay_in_bounds(&mut self.position)
    }
    pub fn pos(&self) -> Vect {
        self.position
    }
    pub fn vel(&self) -> Vect {
        self.velocity
    }
}

impl RotatableObject {
    pub fn new(s: Vect, v: Vect, rot: f32) -> Self {
        RotatableObject {
            physics_obj: PhysicsObject {
                position: s,
                velocity: v,
                acceleration: Vector2(0., 0.),
            },
            rotation: rot,
        }
    }
}

impl Deref for RotatableObject {
    type Target = PhysicsObject;
    fn deref(&self) -> &Self::Target {
        &self.physics_obj
    }
}

impl DerefMut for RotatableObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.physics_obj
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Planet {
    pub obj: PhysicsObject,
    pub health: u8
}

#[derive(Debug, Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Player{
    pub obj: RotatableObject,
    pub health: u8
}

impl Default for Player {
    fn default() -> Self {
        Player {
            obj: Default::default(),
            health: 5
        }
    }
}

impl Planet {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Planet {
            obj: PhysicsObject::new(x, y, vx, vy),
            health: 5
        }
    }
}

use rand::{Rand, Rng};

impl Rand for Planet {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let v = (rng.gen_range(-100., 100.), rng.gen_range(-100., 100.));
        Planet::new(rng.gen_range(-W, W), rng.gen_range(-H, H), v.0, v.1)
    }
}

const W: f32 = 1200./2.;
const H: f32 =  900./2.;

/// Wraps `p` if out of bounds
fn stay_in_bounds(p: &mut Vect) -> bool {
    let mut out_of_bounds;
    if p.0 < -W {
        p.0 += 2. * W;
        out_of_bounds = true;
    } else if p.0 > W {
        p.0 -= 2. * W;
        out_of_bounds = true;
    } else {
        out_of_bounds = false;
    }
    if p.1 < -H {
        p.1 += 2. * H;
        out_of_bounds &= true;
    } else if p.1 > H {
        p.1 -= 2. * H;
        out_of_bounds &= true;
    }
    out_of_bounds
}
