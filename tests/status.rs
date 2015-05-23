extern crate mpd;

use std::net::TcpStream;

#[test]
fn test_status() {
    let mut mpd = mpd::Client::new(TcpStream::connect("192.168.1.10:6600").unwrap()).unwrap();
    let status = mpd.status().unwrap();
    println!("{:?}", status);
}
