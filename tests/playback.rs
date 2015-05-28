extern crate mpd;
extern crate time;

mod helpers;

#[test]
fn playback() {
    let mut mpd = helpers::connect();
    mpd.play().unwrap();
}
