extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
fn playlists() {
    let mut mpd = connect();
    let pls = mpd.playlists().unwrap();
    println!("{:?}", pls);

    for pl in &pls {
        println!("{}: {:?}", pl.name, mpd.playlist(&pl.name).unwrap());
    }
}
