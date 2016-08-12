extern crate mpd;

use std::net::TcpStream;
use mpd::{Client, Query};
//use mpd::playlists::MpdPlaylist;
//use mpd::outputs::MpdOutput;
//use mpd::idle::{MpdEvent, PLAYER, UPDATE};
//use rustc_serialize::json;
//use std::str::from_str;

fn main() {
    let mut c = Client::new(TcpStream::connect("127.0.0.1:6600").unwrap()).unwrap();
    println!("version: {:?}", c.version);
    println!("status: {:?}", c.status());

    for song in c.find(Query::new().and("artist", "Enigma").and("genre", "New Age"), false).unwrap() {
        println!("{:?}", song);
    }
    println!("count: {:?}", c.count(Query::new().and("artist", "Enigma").and("genre", "New Age")).unwrap());
    //println!("stats: {:?}", c.stats());
    ////println!("song: {:?}", c.current_song());
    //println!("queue: {:?}", c.queue());
    //println!("outputs: {:?}", c.outputs());

    //println!("playlists:");
    //for pl in c.playlists().unwrap().iter() {
        //println!("{:?}", pl);
        //println!("{:?}", pl.songs(&mut c));
    //}

    ////let conn = match c {
        ////None => panic!("connection is None"),
        ////Some(Err(e)) => panic!("connection error: {}", e),
        ////Some(Ok(c)) => c
    ////};

    //////println!("{}", json::encode(&conn.status()));
    //////println!("{}", json::encode(&conn.stats()));
    ////println!("{}", json::encode(&conn.playlists()));
    ////println!("{}", json::encode(&conn.outputs()));
    //////println!("{}", json::encode(&conn.queue().songs()));

    ////for mut out in conn.outputs().unwrap().map(|v| v.unwrap()) {
        ////println!("enabling {}", out.enable(true));
    ////}

    //////println!("{}", conn.stop());

    //////println!("{}", conn.current_song());

    ////let v = conn.version();
    ////println!("version: {}", v);

    //////for e in conn.wait(None) {
        //////match e {
            //////Err(ref v) => panic!("{}", v),
            //////Ok(ref v) => println!("{}", v)
        //////}
    //////}
}
