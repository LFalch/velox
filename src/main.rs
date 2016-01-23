#[macro_use]
extern crate korome;

use korome::*;

mod obj;
use obj::*;

fn main() {
    let graphics = Graphics::new("SPACE-SHOOTER", 1200, 900);

    let planet = include_texture!(graphics, "planet.png").unwrap();
    let player = include_texture!(graphics, "ship.png"  ).unwrap();

    let mut objs = Vec::new();
    objs.push(new_player(&player));

    let mut gm = GameManager::new(graphics);

    'main: while let Some((info, mut drawer)) = gm.next_frame(){
        for ke in info.get_key_events(){
            if let (false, VirtualKeyCode::Escape) = *ke{
                break 'main
            }
        }

        for me in info.get_mouse_events(){
            if let (false, MouseButton::Left) = *me {
                objs.push(Obj::new(&planet, info.mousepos.into()));
                println!("Object added, new count: {}", objs.len());
            }
        }

        drawer.clear(0., 0., 0.);
        for obj in objs.iter_mut().rev(){
            obj.update(&info);
            obj.draw(&mut drawer).unwrap()
        }
    }
}
