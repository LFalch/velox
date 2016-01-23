use korome::*;

pub struct Obj<'a>{
    pub rot: f32,
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
    pub tex: &'a Texture,
}

impl<'a> Obj<'a>{
    pub fn new(tex: &'a Texture) -> Self{
        Obj{
            rot: 0.,
            pos: Vector2(0., 0.),
            vel: Vector2(0., 0.),
            tex: tex,
        }
    }
}

impl<'a> Update for Obj<'a>{
    #[inline]
    fn update(&mut self, info: &FrameInfo){
        self.pos = self.pos + self.vel * info.delta as f32;
    }
}

impl<'a> Draw for Obj<'a>{
    #[inline]
    fn draw(&self, drawer: &mut Drawer) -> DrawResult<()>{
        let (x, y) = self.pos.into();
        drawer.draw_texture(self.tex, x, y, self.rot)
    }
}

pub struct Player<'a>{
    inner: Obj<'a>
}

impl<'a> Player<'a>{
    #[inline]
    pub fn new(tex: &'a Texture) -> Self{
        Player{
            inner: Obj::new(tex)
        }
    }
}

impl<'a> Update for Player<'a>{
    #[inline]
    fn update(&mut self, info: &FrameInfo){
        let delta = info.delta as f32;

        let mut acceleration = 0.;
        let ref mut player = self.inner;

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

        player.update(info);
    }
}

impl<'a> Draw for Player<'a>{
    #[inline]
    fn draw(&self, drawer: &mut Drawer) -> DrawResult<()>{
        self.inner.draw(drawer)
    }
}
