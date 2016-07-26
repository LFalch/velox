use ::korome::*;
use ::obj::*;

pub struct SpaceShooter<'a>{
    objs: ObjSystem<'a>,
    sun   : &'a Texture,
    planet: &'a Texture,
    arrow : &'a Texture
}

impl<'a> SpaceShooter<'a>{
    pub fn new(planet: &'a Texture, player: &'a Texture, sun: &'a Texture, arrow: &'a Texture) -> Self{
        let mut objs = ObjSystem::new();
        objs.0.push(new_player(player));

        SpaceShooter{
            sun: sun,
            objs: objs,
            planet: planet,
            arrow: arrow
        }
    }

    fn add_planet(&mut self, pos: Vector2<f32>){
        self.objs.0.push(Object::new(self.planet, pos, 64., 1e13));
    }

    fn add_sun(&mut self, pos: Vector2<f32>){
        self.objs.0.push(Object::new(self.sun, pos, 64., 2e14));
    }
}

impl<'a> Game for SpaceShooter<'a>{
    fn frame(&mut self, info: FrameInfo, mut drawer: Drawer) -> GameUpdate{
        for ke in info.get_key_events(){
            if let (false, VirtualKeyCode::Escape) = *ke{
                return GameUpdate::nothing().set_close(true)
            }
        }

        for me in info.get_mouse_events(){
            match *me{
                (false, MouseButton::Left) => {
                    self.add_planet(info.mousepos.into());
                    println!("Object added, new count: {}", self.objs.0.len());
                },
                (false, MouseButton::Right) => {
                    self.add_sun(info.mousepos.into());
                    println!("Object added, new count: {}", self.objs.0.len());
                },
                _ => ()
            }
        }

        drawer.clear(0., 0., 0.);
        self.objs.update(&info, &mut drawer, self.arrow);

        GameUpdate::nothing()
    }
}

pub struct ObjSystem<'a>(Vec<Object<'a>>);

const G: f32 = 6.671281903963040991511534289e-11;
#[allow(dead_code)]
const G_OLD: f32 = 6.67384E-11;

impl<'a> ObjSystem<'a>{
    #[inline]
    pub fn new() -> Self{
        ObjSystem(Vec::new())
    }

    pub fn update(&mut self, info: &FrameInfo, drawer: &mut Drawer, arrow: &Texture){
        let things = self.all_inners();

        for (i, x) in self.0.iter_mut().enumerate(){
            let mut net_grav = Vector2(0., 0.);

            for y in things.iter().enumerate().filter_map(|(j, a)| if i != j{Some(a)}else{None}){
                let g_force = (G * x.mass * y.mass) / 1024.;
                let dir_towards_body = x.pos.direction_to(y.pos);

                let force = Vector2::unit_vector(dir_towards_body) * g_force;
                net_grav += force;
            }
            x.update(info, net_grav);
            x.draw(drawer, arrow, net_grav, net_grav);
        }
    }

    pub fn all_inners(&self) -> Vec<InnerObject>{
        self.0.iter().map(|x| **x).collect()
    }
}
