use korome::*;
use korome::easy::*;

pub struct InnerObject{
    pub rot: f32,
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
}

pub struct Object<'a>{
    inner: InnerObject,
    tex: &'a Texture,
    update_f: Box<FnMut(&mut InnerObject, &FrameInfo)>,
}

impl<'a> Object<'a>{
    #[inline]
    pub fn new(tex: &'a Texture, pos: Vector2<f32>) -> Self{
        Object::with_update(tex, |_, _| (), pos)
    }

    pub fn with_update<F: 'static + FnMut(&mut InnerObject, &FrameInfo)>(tex: &'a Texture, update: F, pos: Vector2<f32>) -> Self{
        Object{
            tex: tex,
            update_f: Box::new(update),
            inner: InnerObject{
                rot: 0.,
                pos: pos,
                vel: Vector2(0., 0.),
            }
        }
    }
}

const W: f32 = super::WIDTH  as f32 / 2.;
const H: f32 = super::HEIGHT as f32 / 2.;

impl<'a> Obj for Object<'a>{
    #[inline]
    fn update(&mut self, info: &FrameInfo){
        (self.update_f)(&mut self.inner, info);

        self.inner.pos += self.inner.vel * info.delta as f32;

        let Vector2(ref mut x, ref mut y) = self.inner.pos;

        if *x < -W {
            *x += W * 2.
        }
        if *x > W {
            *x -= W * 2.
        }
        if *y < -H {
            *y += H * 2.
        }
        if *y > H {
            *y -= H * 2.
        }
    }
    #[inline]
    fn draw(&self, drawer: &mut Drawer){
        let (x, y) = self.inner.pos.into();
        drawer.draw_texture(self.tex, x, y, self.inner.rot).unwrap()
    }
}

#[inline]
pub fn new_player(tex: &Texture) -> Object{
    Object::with_update(tex, player_update, Vector2(0., 0.))
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
