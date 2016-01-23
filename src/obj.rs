use korome::*;

pub struct InnerObj{
    pub rot: f32,
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
}

pub struct Obj<'a>{
    inner: InnerObj,
    tex: &'a Texture,
    update_f: Box<FnMut(&mut InnerObj, &FrameInfo)>,
}

impl<'a> Obj<'a>{
    #[inline]
    pub fn new(tex: &'a Texture, pos: Vector2<f32>) -> Self{
        Obj::with_update(tex, |_, _| (), pos)
    }

    pub fn with_update<F: 'static + FnMut(&mut InnerObj, &FrameInfo)>(tex: &'a Texture, update: F, pos: Vector2<f32>) -> Self{
        Obj{
            tex: tex,
            update_f: Box::new(update),
            inner: InnerObj{
                rot: 0.,
                pos: pos,
                vel: Vector2(0., 0.),
            }
        }
    }
}

impl<'a> Update for Obj<'a>{
    #[inline]
    fn update(&mut self, info: &FrameInfo){
        (self.update_f)(&mut self.inner, info);
        self.inner.pos = self.inner.pos + self.inner.vel * info.delta as f32;
    }
}

impl<'a> Draw for Obj<'a>{
    #[inline]
    fn draw(&self, drawer: &mut Drawer) -> DrawResult<()>{
        let (x, y) = self.inner.pos.into();
        drawer.draw_texture(self.tex, x, y, self.inner.rot)
    }
}

pub fn new_player(tex: &Texture) -> Obj{
    Obj::with_update(tex, player_update, Vector2(0., 0.))
}

fn player_update(player: &mut InnerObj, info: &FrameInfo){
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

    player.vel = player.vel + Vector2::<f32>::unit_vector(player.rot) * acceleration;
}
