extern crate mpd;

use mpd::Client;
use std::net::TcpStream;

fn main() {
    let mut c = Client::new(TcpStream::connect("127.0.0.1:6600").unwrap()).unwrap();
    println!("version: {:?}", c.version);
    println!("status: {:?}", c.status());
}
