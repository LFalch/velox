extern crate velox;
extern crate rand;

mod serv;

fn main() {
    serv::Server::new().run()
}
