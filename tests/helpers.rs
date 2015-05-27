extern crate mpd;

use std::net::TcpStream;
use std::env;

pub fn connect() -> mpd::Client<TcpStream> {
    let addr = env::var("MPD_SOCK").unwrap_or("127.0.0.1:6600".to_owned());
    mpd::Client::<TcpStream>::connect(&*addr).unwrap()
}

