extern crate mpd;

mod helpers;

#[test]
fn idle() {
    let mut mpd = helpers::connect();
    let idle = mpd.idle(&[]).unwrap();

    let mut mpd1 = helpers::connect();
    mpd1.volume(0).unwrap();

    let sys = idle.get().unwrap();
    assert_eq!(&*sys, &[mpd::Subsystem::Mixer]);
}
