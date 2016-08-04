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
    deltas: Option<Vec<f64>>,
}

use std::ops::Drop;

impl<'a> Drop for SpaceShooter<'a>{
    fn drop(&mut self){
        if let Some(ref mut deltas) = self.deltas{
            use std::fs::File;
            use std::io::{Write, BufWriter};

            let file = File::create("deltas.log").unwrap();
            let mut writer = BufWriter::new(file);

            let (len, mut min, mut max, mut sum) = (deltas.len(), ::std::f64::MAX, 0f64, 0.);

            // Skip the first two 'cause they're normally too high.
            for n in deltas.drain(..).skip(2){
                min = min.min(n);
                max = max.max(n);
                sum += n;

                writer.write_fmt(format_args!("{}s ~ {}fps\n", n, 1./n)).unwrap()
            }

            let avg = sum/len as f64;

            println!("\nTotal  : {}s", sum);
            println!("Best   : {}s ~ {}fps", min, 1./min);
            println!("Worst  : {}s ~ {}fps", max, 1./max);
            println!("Average: {}s ~ {}fps", avg, 1./avg);
        }
    }
}

impl<'a> SpaceShooter<'a>{
    pub fn new(deltas: bool, planet: &'a Texture, player: &'a Texture, sun: &'a Texture, arrow: &'a Texture, laser: &'a Texture) -> Self{
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
            deltas: if deltas{Some(Vec::with_capacity(10_000))}else{None},
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
        self.deltas.as_mut().map(|d| d.push(info.delta));

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

                println!("{}::{}:{}:{}", self.deltas.as_ref().map(Vec::capacity).unwrap_or_default(), a.capacity(), b.capacity(), c.capacity());
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
    if !force.is_nan(){
        force
    }else{
        Vector2(0., 0.)
    }
}

fn inners(x: &Vec<Object>) -> Vec<InnerObject>{
    x.iter().map(|x| **x).collect()
}

impl<'a> ObjSystem<'a>{
    #[inline]
    pub fn new() -> Self{
        Default::default()
    }

    pub fn update(&mut self, info: &FrameInfo, drawer: &mut Drawer, arrow: Option<&Texture>, oobb: OutOfBoundsBehaviour){
        let bs = inners(&self.bodies);
        let wh = drawer.graphics.get_h_size();
        let delta = info.delta as f32;

        let (mut acceleration, mut rot) = (0., 0.);
        is_down!{info;
            D, Right => {
                rot -= 2. * delta
            },
            A, Left => {
                rot += 2. * delta
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

        let mut indices_to_die = [Vec::new(), Vec::new(), Vec::new()];

        for (i, player) in self.players.iter_mut().enumerate(){
            let net_grav = bs.iter().fold(Vector2(0., 0.), |n_g, y| n_g + gravity(&player, y));

            let propulsion_force = Vector2::<f32>::unit_vector(player.rot) * acceleration;

            player.rot += rot;
            player.vel += propulsion_force;

            player.update(info, net_grav, wh, oobb);
            player.draw(drawer, arrow, net_grav, net_grav+propulsion_force);

            if player.health == 0{
                indices_to_die[0].push(i);
            }
        }
        for (i, body) in self.bodies.iter_mut().enumerate(){
            let net_grav = bs.iter().enumerate().filter_map(|(j, y)| if i==j{None}else{Some(y)}).fold(Vector2(0., 0.), |n_g, y| n_g + gravity(&body, y));

            body.update(info, net_grav, wh, oobb);
            body.draw(drawer, arrow, net_grav, net_grav);

            if body.health == 0{
                indices_to_die[1].push(i);
            }
        }
        for (i, projectile) in self.projectiles.iter_mut().enumerate(){
            let net_grav = bs.iter().fold(Vector2(0., 0.), |n_g, y| n_g + gravity(&projectile, y));

            projectile.rot = projectile.vel.direction();

            projectile.update(info, net_grav, wh, oobb);
            projectile.draw(drawer, arrow, net_grav, net_grav);

            for player in self.players.iter_mut(){
                if projectile.pos.distance_to(player.pos) - player.diameter/2. <= 16.{
                    player.health = player.health.saturating_sub(1);
                    indices_to_die[2].push(i);
                }
            }
            for body in self.bodies.iter_mut(){
                if projectile.pos.distance_to(body.pos) - body.diameter/2. <= 16.{
                    body.health = body.health.saturating_sub(1);
                    indices_to_die[2].push(i);
                }
            }
        }
        for v in &mut indices_to_die{
            v.sort();
            v.dedup();
        }
        for &i in indices_to_die[0].iter().rev(){
            self.players.remove(i);
        }
        for &i in indices_to_die[1].iter().rev(){
            self.bodies.remove(i);
        }
        for &i in indices_to_die[2].iter().rev(){
            self.projectiles.remove(i);
        }
    }

    pub fn len(&self) -> usize{
        self.players.len() + self.bodies.len() + self.projectiles.len()
    }
}
