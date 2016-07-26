use std::ops::{Deref, DerefMut};

use korome::*;

#[derive(Debug, Copy, Clone)]
pub struct InnerObject{
    pub radius: f32,
    pub rot: f32,
    pub mass: f32,
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
}

pub struct Object<'a>{
    inner: InnerObject,
    tex: &'a Texture,
    update_f: Box<FnMut(&mut InnerObject, &FrameInfo)>,
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
    pub fn new(tex: &'a Texture, pos: Vector2<f32>, radius: f32, mass: f32) -> Self{
        Object::with_update(tex, |_, _| (), pos, radius, mass)
    }

    pub fn with_update<F>(tex: &'a Texture, update: F, pos: Vector2<f32>, radius: f32, mass: f32) -> Self
    where F: 'static + FnMut(&mut InnerObject, &FrameInfo){
        Object{
            tex: tex,
            update_f: Box::new(update),
            inner: InnerObject{
                radius: radius,
                mass: mass,
                rot: 0.,
                pos: pos,
                vel: Vector2(0., 0.),
            }
        }
    }

    #[inline]
    pub fn update(&mut self, info: &FrameInfo, force: Vector2<f32>, (w, h): (f32, f32), oobb: OutOfBoundsBehaviour){
        (self.update_f)(&mut self.inner, info);

        let &mut InnerObject{ref mut pos, ref mut vel, mass, ..} = self.deref_mut();

        *pos += *vel * info.delta as f32;
        *vel += force / mass;

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
    #[inline]
    pub fn draw(&self, drawer: &mut Drawer, arrow: Option<&Texture>, net_grav: Vector2<f32>, net_force: Vector2<f32>){
        let vel = self.vel;
        let pos = self.pos;

        // draw_blue_circle(radius, self.inner.pos);

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
                //Draws a green arrow pointing in the direction of the net gravitational force being put on the body.
                let mut arrow_vec = net_grav.normalise() * 48.;
                arrow_vec.1 *= 1.;
                arrow_drawer.clone()
                .pos((arrow_vec + pos).into())
                .rotation(net_grav.direction())
                .colour([0., 1., 0., 1.])
                .draw(drawer);
            }
            if net_force.length() != 0.{
                //Draws a blue arrow pointing in the direction of the net force being put on the body.
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
    Object::with_update(tex, player_update, Vector2(0., 0.), 10., 32.)
}

pub fn laser_update(laser: &mut InnerObject, _info: &FrameInfo){
    laser.rot = laser.vel.direction();
}

fn player_update(player: &mut InnerObject, info: &FrameInfo){
    let delta = info.delta as f32;

    let mut acceleration = 0.;

    is_down!{info;
        D, Right => {
            player.rot -= 2. * delta
        },
        A, Left => {
            player.rot += 2. * delta
        },
        S, Down => {
            acceleration -= 52. * delta
        },
        W, Up => {
            acceleration += 52. * delta;
        },
        LShift => {
            acceleration *= 7.;
        }
    }

    player.vel += Vector2::<f32>::unit_vector(player.rot) * acceleration;
}
