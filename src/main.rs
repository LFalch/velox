#[macro_use]
extern crate korome;
extern crate glium;

use korome::*;

mod obj;
use obj::*;

fn main() {
    let title = format!("SPACE-SHOOTER (korome {})", korome::VERSION);

    let draw = Draw::new(&title, 800, 600);

    let planet = include_texture!(draw, "planet.png", 64, 64).unwrap();
    let player = include_texture!(draw, "ship.png"  , 32, 32).unwrap();

    let mut objs = Vec::new();

    objs.push(Obj{
        rot: 0.,
        pos: Vector2(0., 0.),
        vel: Vector2(0., 0.),
        tex: &planet
    });

    objs.push(Obj{
        rot: 0.,
        pos: Vector2(0., 0.),
        vel: Vector2(0., 0.),
        tex: &player
    });

    let game = Game::with_shared(draw, objs, logic, render);

    game.run_until_closed();
}

pub fn logic (objs: &mut Vec<Obj>, logic_args: LogicArgs){
    {
        let delta = logic_args.delta as f32;

        let ref mut player = objs[1];

        let mut acceleration = 0.;

        is_down!{logic_args;
            D, Right => {
                player.rot -= 2. * delta
            },
            A, Left => {
                player.rot += 2. * delta
            },
            S, Down => {
                acceleration -= 3. * delta
            },
            W, Up => {
                acceleration += 3. * delta
            }
        }

        player.vel = player.vel + Vector2::<f32>::unit_vector(player.rot) * acceleration;
    }

    for obj in objs.iter_mut(){
        obj.update()
    }
}

pub fn render(objs: &Vec<Obj>, mut render_args: RenderArgs){
    render_args.draw_sprites(objs).unwrap();
}
