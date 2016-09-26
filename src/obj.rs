use std::ops::{Deref, DerefMut};

use korome::{Texture, FrameInfo, Drawer};
use ::Vector2;

#[derive(Debug, Copy, Clone)]
pub struct InnerObject{
    pub health: u32,
    pub diameter: f32,
    pub rot: f32,
    pub mass: f32,
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
}

pub struct Object<'a>{
    inner: InnerObject,
    tex: &'a Texture,
}

impl<'a> Deref for Object<'a>{
    type Target = InnerObject;
    #[inline(always)]
    fn deref(&self) -> &InnerObject{
        &self.inner
    }
}

impl<'a> DerefMut for Object<'a>{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut InnerObject{
        &mut self.inner
    }
}

use ::OutOfBoundsBehaviour::{self, Wrap, Bounce, Stop};

impl<'a> Object<'a>{
    #[inline]
    pub fn new(tex: &'a Texture, pos: Vector2<f32>, health: u32, diameter: f32, mass: f32) -> Self{
        Object{
            tex: tex,
            inner: InnerObject{
                health: health,
                diameter: diameter,
                mass: mass,
                rot: 0.,
                pos: pos,
                vel: Vector2(0., 0.),
            }
        }
    }
    pub fn update(&mut self, info: &FrameInfo, impulse: Vector2<f32>, (w, h): (f32, f32), oobb: OutOfBoundsBehaviour){
        let &mut InnerObject{ref mut pos, ref mut vel, ..} = self.deref_mut();

        *pos += *vel * info.delta;
        *vel += impulse;

        let &mut Vector2(ref mut x, ref mut y) = pos;
        let &mut Vector2(ref mut vx, ref mut vy) = vel;

        match oobb{
            Wrap => {
                if *x < -w {
                    *x += w * 2.
                }
                if *x > w {
                    *x -= w * 2.
                }
                if *y < -h {
                    *y += h * 2.
                }
                if *y > h {
                    *y -= h * 2.
                }
            },
            Bounce => {
                if *x < -w {
                    *vx = vx.abs()
                }
                if *x > w {
                    *vx = -vx.abs()
                }
                if *y < -h {
                    *vy = vy.abs()
                }
                if *y > h {
                    *vy = -vy.abs()
                }
            },
            Stop => {
                if *x < -w {
                    *x = -w;
                    *vx = 0.;
                }
                if *x > w {
                    *x =  w;
                    *vx = 0.;
                }
                if *y < -h {
                    *y = -h;
                    *vy = 0.;
                }
                if *y > h {
                    *y =  h;
                    *vy = 0.;
                }
            }
        }
    }
    pub fn draw(&self, drawer: &mut Drawer, arrow: Option<&Texture>, net_grav: Vector2<f32>, net_force: Vector2<f32>){
        let vel = self.vel;
        let pos = self.pos;

        // draw_blue_circle(diameter, self.inner.pos);

        self.tex.drawer()
            .pos(pos.into())
            .rotation(self.rot)
            .draw(drawer);

        if let Some(arrow) = arrow{
            let arrow_drawer = arrow.drawer();

            if vel.length() != 0.{
                //Draws a red arrow pointing in the direction of the velocity.
                let mut arrow_vec = vel.normalise() * 40.;
                arrow_vec.1 *= 1.;
                arrow_drawer.clone()
                .pos((arrow_vec + pos).into())
                .rotation(vel.direction())
                .colour([1., 0., 0., 1.])
                .draw(drawer);
            }
            if net_grav.length() != 0.{
                //Draws a green arrow pointing in the direction of the net gravitational force being put on the Object.
                let mut arrow_vec = net_grav.normalise() * 48.;
                arrow_vec.1 *= 1.;
                arrow_drawer.clone()
                .pos((arrow_vec + pos).into())
                .rotation(net_grav.direction())
                .colour([0., 1., 0., 1.])
                .draw(drawer);
            }
            if net_force.length() != 0.{
                //Draws a blue arrow pointing in the direction of the net force being put on the Object.
                let mut arrow_vec = net_force.normalise() * 32.;
                arrow_vec.1 *= 1.;
                arrow_drawer.clone()
                .pos((arrow_vec + pos).into())
                .rotation(net_force.direction())
                .colour([0., 0., 1., 1.])
                .draw(drawer);
            }
        }
    }
}

#[inline]
pub fn new_player(tex: &Texture) -> Object{
    Object::new(tex, Vector2(0., 0.), 3, 10., 32.)
}

pub fn new_laser(tex: &Texture, pos: Vector2<f32>, direction: f32) -> Object{
    let unit = Vector2::unit_vector(direction);
    let mut laser = Object::new(tex, pos + unit * 32., 3, 32., 1e-10);
    {
        let &mut InnerObject{ref mut vel, ref mut rot,..} = laser.deref_mut();
        *vel = Vector2::unit_vector(direction) * 800.;
        *rot = direction;
    }
    laser
}
