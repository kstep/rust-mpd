extern crate mpd;

mod helpers;
use std::time::Duration;

use helpers::connect;
use mpd::Song;

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
fn lsinfo() {
    let mut mpd = connect();
    let songs = mpd.lsinfo(Song { file: "silence.flac".into(), ..Default::default() }).unwrap();
    assert_eq!(songs.len(), 1);

    let song = songs.get(0).unwrap();
    assert_eq!(song.file, "silence.flac");
    assert_eq!(song.duration.expect("song should have duration"), Duration::from_millis(500));
}

#[test]
fn rescan_update() {
    let mut mpd = connect();
    println!("update: {:?}", mpd.update());
    println!("rescan: {:?}", mpd.rescan());
}
