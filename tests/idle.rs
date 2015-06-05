extern crate mpd;

mod helpers;

use mpd::Idle;

#[test]
fn idle() {
    let mut mpd = helpers::connect();
    let idle = mpd.idle(&[]).unwrap();

    let mut mpd1 = helpers::connect();
    mpd1.consume(true).unwrap();
    mpd1.consume(false).unwrap();

    let sys = idle.get().unwrap();
    assert_eq!(&*sys, &[mpd::Subsystem::Options]);
}
