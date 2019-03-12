extern crate mpd;

use mpd::{Client, Query};

fn main() {
    let mut c = Client::connect("127.0.0.1:6600").unwrap();
    println!("version: {:?}", c.version);
    println!("status: {:?}", c.status());
    println!("stuff: {:?}", c.find(&Query::new(), (1, 2)));
}
