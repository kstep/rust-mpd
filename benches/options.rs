#![feature(test)]

extern crate mpd;
extern crate time;
extern crate test;
extern crate unix_socket;

use test::{Bencher, black_box};
use unix_socket::UnixStream;

#[bench]
fn status(b: &mut Bencher) {
    let mut mpd = mpd::Client::<UnixStream>::new(UnixStream::connect("/var/run/mpd/socket").unwrap()).unwrap();
    b.iter(|| { black_box(mpd.status()).unwrap(); });
}
