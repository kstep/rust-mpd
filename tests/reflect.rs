extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
fn commands() {
    let mut mpd = connect();
    println!("{:?}", mpd.commands().unwrap());
}

#[test]
fn urlhandlers() {
    let mut mpd = connect();
    println!("{:?}", mpd.urlhandlers().unwrap());
}

#[test]
fn decoders() {
    let mut mpd = connect();
    println!("{:?}", mpd.decoders().unwrap());
}

#[test]
fn tagtypes() {
    let mut mpd = connect();
    println!("{:?}", mpd.tagtypes().unwrap());
}
