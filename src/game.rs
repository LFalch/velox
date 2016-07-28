use ::korome::*;
use ::obj::*;

use std::ops::Deref;

pub struct SpaceShooter<'a>{
    objs: ObjSystem<'a>,
    player: &'a Texture,
    sun   : &'a Texture,
    planet: &'a Texture,
    arrow : &'a Texture,
    laser : &'a Texture,
    oobb  : OutOfBoundsBehaviour,
    dbg_a : bool,
    deltas: Vec<f64>,
}

use std::ops::Drop;

impl<'a> Drop for SpaceShooter<'a>{
    fn drop(&mut self){
        use std::fs::File;
        use std::io::{Write, BufWriter};

        let file = File::create("deltas.log").unwrap();
        let mut writer = BufWriter::new(file);

        let (len, mut min, mut max, mut sum) = (self.deltas.len(), ::std::f64::MAX, 0f64, 0.);

        for n in self.deltas.drain(..){
            min = min.min(n);
            max = max.max(n);
            sum += n;

            writer.write_fmt(format_args!("{}s ~ {}fps\n", n, 1./n)).unwrap()
        }

        let avg = sum/len as f64;

        println!("Total  : {}s", sum);
        println!("Best   : {}s ~ {}fps", min, 1./min);
        println!("Worst  : {}s ~ {}fps", max, 1./max);
        println!("Average: {}s ~ {}fps", avg, 1./avg);
    }
}

impl<'a> SpaceShooter<'a>{
    pub fn new(planet: &'a Texture, player: &'a Texture, sun: &'a Texture, arrow: &'a Texture, laser: &'a Texture) -> Self{
        let mut objs = ObjSystem::new();
        objs.players.push(new_player(player));

        SpaceShooter{
            player: player,
            sun   : sun,
            objs  : objs,
            planet: planet,
            arrow : arrow,
            laser : laser,
            oobb  : Bounce,
            dbg_a : true,
            deltas: Vec::with_capacity(10_000),
        }
    }

    fn add_player(&mut self){
        self.objs.players.push(new_player(self.player))
    }

    fn add_planet(&mut self, pos: Vector2<f32>){
        self.objs.bodies.push(Object::new(self.planet, pos, 50, 64., 1e13));
    }

    fn add_sun(&mut self, pos: Vector2<f32>){
        self.objs.bodies.push(Object::new(self.sun, pos, 100, 64., 2e14));
    }

    fn add_laser(&mut self, pos: Vector2<f32>, direction: f32){
        self.objs.projectiles.push(new_laser(self.laser, pos, direction));
    }
}

macro_rules! when_released{
    ($info:expr; $($key:ident => $b:block),+) => {
        for ke in $info.get_key_events(){
            match *ke{
                $((false, VirtualKeyCode::$key) => $b,)+
                _ => ()
            }
        }
    };
    ($info:expr;MOUSE; $($key:ident => $b:block),+) => {
        for ke in $info.get_mouse_events(){
            match *ke{
                $((false, MouseButton::$key) => $b,)+
                _ => ()
            }
        }
    };
}

impl<'a> Game for SpaceShooter<'a>{
    fn frame(&mut self, info: FrameInfo, mut drawer: Drawer) -> GameUpdate{
        self.deltas.push(info.delta);

        when_released!{info;
            Escape => {
                return GameUpdate::nothing().set_close(true)
            },
            M => {
                self.oobb = match self.oobb{
                    Bounce => Stop,
                    Stop => Wrap,
                    Wrap => Bounce
                };
                println!("OOBB changed to: {:?}", self.oobb);
            },
            N => {
                self.dbg_a = !self.dbg_a;
                println!("Turned debug arrows {}", if self.dbg_a{"on"}else{"off"});
            },
            C => {
                self.objs.bodies.clear();
                println!("Bodies cleared, new count: {}", self.objs.len());
            },
            K => {
                self.objs.projectiles.clear();
                println!("Projectiles cleared, new count: {}", self.objs.len());
            },
            O => {
                self.objs.players.clear();
                println!("Players cleared, new count: {}", self.objs.len());
            },
            P => {
                self.add_player();
                println!("Player added, new count: {}", self.objs.len());
            },
            Q => {
                let ObjSystem{players: ref a, bodies: ref b, projectiles: ref c} = self.objs;

                println!("{}::{}:{}:{}", self.deltas.capacity(), a.capacity(), b.capacity(), c.capacity());
            },
            Z => {
                let ObjSystem{players: ref mut a, bodies: ref mut b, projectiles: ref mut c} = self.objs;
                print!("{};{};{}->", a.capacity(), b.capacity(), c.capacity());
                a.shrink_to_fit();
                b.shrink_to_fit();
                c.shrink_to_fit();
                println!("{};{};{}", a.capacity(), b.capacity(), c.capacity());
            },
            X => {
                for x in self.objs.players.iter().chain(self.objs.bodies.iter()).chain(self.objs.projectiles.iter()){
                    println!("{:?}", &**x);
                }
            },
            Space => {
                let iter: Vec<_> = self.objs.players.iter().map(|x| {
                    let &InnerObject{pos, rot,..} = x.deref();
                    (pos, rot)
                }).collect();
                for &(pos, rot) in &iter{
                    self.add_laser(pos, rot);
                }
                println!("Laser{} added, new count: {}", if self.objs.players.len() == 1{""}else{"s"}, self.objs.len());
            }
        }

        when_released!{info;MOUSE;
            Left => {
                self.add_planet(info.mousepos.into());
                println!("Object added, new count: {}", self.objs.len());
            },
            Right => {
                self.add_sun(info.mousepos.into());
                println!("Object added, new count: {}", self.objs.len());
            }
        }

        drawer.clear(0., 0., 0.);
        self.objs.update(&info, &mut drawer, if self.dbg_a{Some(self.arrow)}else{None}, self.oobb);

        GameUpdate::nothing()
    }
}

#[derive(Default)]
pub struct ObjSystem<'a>{
    bodies     : Vec<Object<'a>>,
    players    : Vec<Object<'a>>,
    projectiles: Vec<Object<'a>>,
}

const G: f32 = 6.671281903963040991511534289e-11;
#[allow(dead_code)]
const G_OLD: f32 = 6.67384E-11;

use OutOfBoundsBehaviour::*;
use OutOfBoundsBehaviour;

fn gravity(x: &InnerObject, y: &InnerObject) -> Vector2<f32>{
    let r = x.pos.distance_to(y.pos);
    let g_force = (G * x.mass * y.mass) / (r * r);
    let dir_towards_body = x.pos.direction_to(y.pos);

    let force = Vector2::unit_vector(dir_towards_body) * g_force;
    if !(force.0.is_nan() || force.1.is_nan()){
        force
    }else{
        Vector2(0., 0.)
    }
}

impl<'a> ObjSystem<'a>{
    #[inline]
    pub fn new() -> Self{
        Default::default()
    }

    pub fn update(&mut self, info: &FrameInfo, drawer: &mut Drawer, arrow: Option<&Texture>, oobb: OutOfBoundsBehaviour){
        let (_pls, bs, _prs) = self.all_inners();
        let wh = drawer.graphics.get_h_size();
        let delta = info.delta as f32;

        for player in self.players.iter_mut(){
            let net_grav = bs.iter().fold(Vector2(0., 0.), |n_g, y| n_g + gravity(&player, y));

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

            let propulsion_force = Vector2::<f32>::unit_vector(player.rot) * acceleration;

            player.vel += propulsion_force;

            player.update(info, net_grav, wh, oobb);
            player.draw(drawer, arrow, net_grav, net_grav+propulsion_force);
        }
        for (i, body) in self.bodies.iter_mut().enumerate(){
            let net_grav = bs.iter().enumerate().filter_map(|(j, y)| if i==j{None}else{Some(y)}).fold(Vector2(0., 0.), |n_g, y| n_g + gravity(&body, y));

            body.update(info, net_grav, wh, oobb);
            body.draw(drawer, arrow, net_grav, net_grav);
        }
        for projectile in self.projectiles.iter_mut(){
            let net_grav = bs.iter().fold(Vector2(0., 0.), |n_g, y| n_g + gravity(&projectile, y));

            projectile.rot = projectile.vel.direction();

            projectile.update(info, net_grav, wh, oobb);
            projectile.draw(drawer, arrow, net_grav, net_grav);
        }
    }

    pub fn all_inners(&self) -> (Vec<InnerObject>, Vec<InnerObject>, Vec<InnerObject>){
        fn inners(x: &Vec<Object>) -> Vec<InnerObject>{
            x.iter().map(|x| **x).collect()
        }

        (inners(&self.players), inners(&self.bodies), inners(&self.projectiles))
    }

    pub fn len(&self) -> usize{
        self.players.len() + self.bodies.len() + self.projectiles.len()
    }
}
