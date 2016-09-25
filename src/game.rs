use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate};
use simple_vector2d::Vector2;

pub type Vect = Vector2<f32>;

pub struct SpaceShooterBuilder<'a>{
    pub planet: &'a Texture,
    pub ship  : &'a Texture,
    pub sun   : &'a Texture,
    pub arrow : &'a Texture,
    pub laser : &'a Texture,
}

impl<'a> SpaceShooterBuilder<'a>{
    pub fn finish(self) -> SpaceShooter<'a>{
        let SpaceShooterBuilder{planet,ship,sun,arrow,laser} = self;
        SpaceShooter{
            planet: planet,
            ship: ship,
            sun: sun,
            arrow: arrow,
            laser: laser,
            new_planet: None,
            planets: Vec::new()
        }
    }
}

pub struct SpaceShooter<'a>{
    pub planet: &'a Texture,
    pub ship  : &'a Texture,
    pub sun   : &'a Texture,
    pub arrow : &'a Texture,
    pub laser : &'a Texture,
    new_planet: Option<Vect>,
    planets: Vec<(Vect, Vect)>
}

impl<'a> Game for SpaceShooter<'a> {
    fn frame(&mut self, info: &FrameInfo, drawer: &mut Drawer) -> GameUpdate {
        when!{info;
            false, Escape => {
                return GameUpdate::Close
            }
        }
        when_mouse!{info;
            true, Left => {
                if self.new_planet.is_none(){
                    self.new_planet = Some(info.mousepos.into());
                }
            },
            false, Left => {
                if let Some(pos) = self.new_planet{
                    self.planets.push((pos, pos-info.mousepos.into()));
                    self.new_planet = None;
                }
            }
        }

        let wh = drawer.graphics.get_h_size();
        drawer.clear(0., 0., 0.);
        for planet in &mut self.planets{
            self.planet.drawer()
                .pos(planet.0.into())
                .draw(drawer);
            planet.0 += planet.1 * info.delta;

            stay_in_bounds(&mut planet.0, wh);
        }
        if let Some(p) = self.new_planet{
            self.planet.drawer()
                .pos(p.into())
                .colour([0.5; 4])
                .draw(drawer);
        }

        GameUpdate::Nothing
    }
}

/// Wraps `p` if out of bounds
fn stay_in_bounds(p: &mut Vect, (w, h): (f32, f32)) {
    if p.0 < -w{
        p.0 += 2. * w;
    }
    if p.0 > w{
        p.0 -= 2. * w;
    }
    if p.1 < -h{
        p.1 += 2. * h;
    }
    if p.1 > h{
        p.1 -= 2. * h;
    }
}
