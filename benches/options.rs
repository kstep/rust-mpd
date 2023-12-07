#![feature(test)]

extern crate mpd;
extern crate test;

use std::os::unix::net::UnixStream;
use test::{black_box, Bencher};

#[bench]
fn status(b: &mut Bencher) {
    let mut mpd = mpd::Client::<UnixStream>::new(UnixStream::connect("/var/run/mpd/socket").unwrap()).unwrap();
    b.iter(|| {
        black_box(mpd.status()).unwrap();
    });
}
