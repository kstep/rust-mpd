#![allow(unused_imports)]

extern crate mpd;
extern crate "rustc-serialize" as rustc_serialize;

use mpd::connection::MpdConnection;
use mpd::playlists::MpdPlaylist;
use mpd::outputs::MpdOutput;
use mpd::idle::{MpdEvent, PLAYER, UPDATE};
use rustc_serialize::json;
use std::str::from_str;

fn main() {
    //let c = MpdConnection::new(Some("192.168.1.10"), 6600);
    let c = MpdConnection::new(None, 6600);
    let conn = match c {
        None => panic!("connection is None"),
        Some(Err(e)) => panic!("connection error: {}", e),
        Some(Ok(c)) => c
    };

    //println!("{}", json::encode(&conn.status()));
    //println!("{}", json::encode(&conn.stats()));
    println!("{}", json::encode(&conn.playlists()));
    println!("{}", json::encode(&conn.outputs()));
    //println!("{}", json::encode(&conn.queue().songs()));

    for mut out in conn.outputs().unwrap().map(|v| v.unwrap()) {
        println!("enabling {}", out.enable(true));
    }

    //println!("{}", conn.stop());

    //println!("{}", conn.current_song());

    let v = conn.version();
    println!("version: {}", v);

    //for e in conn.wait(None) {
        //match e {
            //Err(ref v) => panic!("{}", v),
            //Ok(ref v) => println!("{}", v)
        //}
    //}
}
