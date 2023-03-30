extern crate mpd;

mod helpers;

#[test]
fn playback() {
    let mut mpd = helpers::connect();
    mpd.play().unwrap();
}
