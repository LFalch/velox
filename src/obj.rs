use korome::*;

pub struct Obj<'a>{
    pub rot: f32,
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
    pub tex: &'a Texture,
}

impl<'a> Obj<'a>{
    pub fn update(&mut self){
        self.pos = self.pos + self.vel;
    }
}

impl<'a> Sprite for Obj<'a>{
    fn get_texture(&self) -> &Texture{
        self.tex
    }

    fn get_pos(&self) -> (f32, f32){
        self.pos.into()
    }

    fn get_rotation(&self) -> f32{
        self.rot
    }
}
