extern crate velox_core;
extern crate rand;

mod serv;

fn main() {
    serv::Server::new().run()
}
