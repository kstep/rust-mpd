extern crate mpd;

use mpd::{Client, Query};
use std::net::TcpStream;

fn main() {
    let mut c = Client::new(TcpStream::connect("127.0.0.1:6600").unwrap()).unwrap();
    println!("version: {:?}", c.version);
    println!("status: {:?}", c.status());
    println!("stuff: {:?}", c.find(&Query::new(), (1, 2)));
}
