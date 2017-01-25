extern crate mpd;

use std::env;
use std::net::TcpStream;

pub fn connect() -> mpd::Client<TcpStream> {
    let addr = env::var("MPD_SOCK").unwrap_or_else(|_| "127.0.0.1:6600".to_owned());
    mpd::Client::connect(&*addr).unwrap()
}
