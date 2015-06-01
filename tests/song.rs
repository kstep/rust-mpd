extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
fn currentsong() {
    let mut mpd = connect();
    let song = mpd.currentsong().unwrap();
    println!("{:?}", song);
}

#[test]
fn queue() {
    let mut mpd = connect();
    let queue = mpd.queue().unwrap();
    println!("{:?}", queue);

    let songs = mpd.songs(..).unwrap();
    assert_eq!(songs, queue);
}

#[test]
fn rescan_update() {
    let mut mpd = connect();
    println!("update: {:?}", mpd.update());
    println!("rescan: {:?}", mpd.rescan());
}
